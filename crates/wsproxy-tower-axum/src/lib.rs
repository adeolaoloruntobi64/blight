use std::sync::Arc;

use appstate::AppState;
use axum::{body::HttpBody, extract::Request, routing::{self, RouterIntoService}, Router};
use config::{ArcedWsProxyServerInfo, WsProxyServerConfig};
use dns::WsProxyDnsResolverWrapper;
use tower::{BoxError, Service};

mod dns;
mod appstate;
mod wsproxy;

pub mod config;

pub struct WsProxyServerService<B> {
    service: RouterIntoService<B, ()>
}

impl<B> Clone for WsProxyServerService<B> {
    fn clone(&self) -> Self {
        Self { service: self.service.clone() }
    }
}

impl<B> WsProxyServerService<B> {
    pub fn new(config: WsProxyServerConfig) -> Self {
        WsProxyServerService {
            service: Router::new()
                .route("/{param}", routing::any(wsproxy::proxy))
                .with_state(AppState::new(
                    ArcedWsProxyServerInfo { inner: Arc::new(config.info) },
                    WsProxyDnsResolverWrapper { resolver: config.dns }
                ))   
                .into_service()
        }
    }
}

impl<B> Service<Request<B>> for WsProxyServerService<B>
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