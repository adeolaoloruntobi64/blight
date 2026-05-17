mod v1;
mod v2;
mod v3;
mod dns;
mod err;
mod util;
mod consts;
mod structs;
mod appstate;
pub mod config;

use std::sync::Arc;

use appstate::AppState;
use axum::{body::HttpBody, extract::Request, routing::{self, RouterIntoService}, Router};
use config::{ArcedBareServerInfo, BareServerConfig};
use dns::TompDnsResolverWrapper;
use tower::Service;

pub use structs::BareServerVersion;
use tower_http::BoxError;

pub struct BareServerService<B> {
    service: RouterIntoService<B, ()>
}

impl<B> Clone for BareServerService<B> {
    fn clone(&self) -> Self {
        Self { service: self.service.clone() }
    }
}

impl<B> BareServerService<B> {
    pub fn new(config: BareServerConfig) -> Self {
        let mut router = Router::new()
            .route("/", routing::get(util::index::request_server_info));

        if config.info.supported_versions.contains(&BareServerVersion::V1) {
            router = router
                .route("/v1/", routing::any(v1::proxy))
                .route("/v1/ws-meta", routing::get(v1::ws_meta::proxy))
                .route("/v1/ws-new-meta", routing::get(v1::ws_new_meta::proxy))
        }

        if config.info.supported_versions.contains(&BareServerVersion::V2) {
            router = router
                .route("/v2/", routing::any(v2::proxy))
                .route("/v2/ws-meta", routing::get(v2::ws_meta::proxy))
                .route("/v2/ws-new-meta", routing::get(v2::ws_new_meta::proxy))
        }
            
        if config.info.supported_versions.contains(&BareServerVersion::V3) {
            router = router
                .route("/v3/", routing::any(v3::proxy))
        }
            
        BareServerService {
            service: router
                .layer(config.cors.clone())
                .with_state(AppState::new(
                    ArcedBareServerInfo { inner: Arc::new(config.info) },
                    TompDnsResolverWrapper { resolver: config.dns }
                ))
                .into_service()
        }
    }
}

impl<B> Service<Request<B>> for BareServerService<B>
where
    B: HttpBody<Data = axum::body::Bytes> + Send + 'static,
    B::Error: Into<BoxError>
{

    type Response = <Router as Service<Request<B>>>::Response;
    type Error = <Router as Service<Request<B>>>::Error;
    type Future = <Router as Service<Request<B>>>::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        self.service.call(req)
    }

}

