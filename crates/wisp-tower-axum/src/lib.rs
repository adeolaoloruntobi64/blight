use std::sync::Arc;

use appstate::AppState;
use axum::{body::HttpBody, extract::Request, routing::{self, RouterIntoService}, Router};
use config::{ArcedWispServerInfo, WispServerConfig};
use dns::WispDnsResolverWrapper;
use versions::WispServerVersion;
use tower::{BoxError, Service};


mod v1;
mod v2;
mod dns;
mod util;
mod appstate;

pub mod config;
pub mod versions;

pub struct WispServerService<B> {
    service: RouterIntoService<B, ()>
}

impl<B> Clone for WispServerService<B> {
    fn clone(&self) -> Self {
        Self { service: self.service.clone() }
    }
}

impl<B> WispServerService<B> {
    pub fn new(config: WispServerConfig) -> Self {
        let mut router = Router::new();

        if config.info.supported_versions.contains(&WispServerVersion::V1) {
            router = router
                .route("/v1/", routing::any(v1::proxy))
        }

        if config.info.supported_versions.contains(&WispServerVersion::V2) {
            router = router
            .route("/v2/", routing::any(v2::proxy))
        }
            
        WispServerService {
            service: router
                .with_state(AppState::new(
                    ArcedWispServerInfo { inner: Arc::new(config.info) },
                    WispDnsResolverWrapper { resolver: config.dns }
                ))   
                .into_service()
        }
    }
}

impl<B> Service<Request<B>> for WispServerService<B>
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