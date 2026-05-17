use std::{convert::Infallible, path::Path};

use axum::{body::{Body, HttpBody}, error_handling::HandleErrorLayer, extract::Request, http::{Response, StatusCode}, routing::RouterIntoService, Router};
use config::StaticClientConfig;
use num_traits::{PrimInt, Unsigned};
use tower::{BoxError, Service, ServiceBuilder};
use tower_etag_cache::{const_lru_provider::ConstLruProvider, EtagCacheLayer};
use tower_http::{compression::CompressionLayer, services::ServeDir};

pub mod config;

async fn handle_etag_cache_layer_err<T: Into<BoxError>>(err: T) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.into().to_string())
}
pub struct StaticClientService<B> {
    service: RouterIntoService<B, ()>
}

impl<B> Clone for StaticClientService<B> {
    fn clone(&self) -> Self {
        Self { service: self.service.clone() }
    }
}

impl<B> StaticClientService<B> {
    pub fn new<const CAP: usize, P, S, Uint>(config: StaticClientConfig<CAP, P, S, Uint>) -> Self
    where
        P: AsRef<Path>,
        S: Service<Request, Error = Infallible, Response = Response<Body>> + Clone + Send + Sync + 'static,
        S::Future: Send + 'static,
        Uint: PrimInt + Unsigned + Send + 'static
    {
        let mut router = Router::new();

        if let Some(fallback) = config.fallback_service {
            router = router.fallback_service(ServeDir::new(config.path).fallback(fallback));
        } else {
            router = router.fallback_service(ServeDir::new(config.path));
        }

        StaticClientService {
            service: router
                .layer(
                    CompressionLayer::new()
                        .gzip(true)
                        .deflate(true)
                        .br(true)
                        .zstd(true)
                )
                .layer(
                    ServiceBuilder::new()
                        .layer(HandleErrorLayer::new(handle_etag_cache_layer_err))
                        .layer(EtagCacheLayer::with_default_predicate(
                            ConstLruProvider::<_, _, CAP, Uint>::init(config.channel_size)
                        ))
                )
                .into_service()
        }
    }
}

impl<B> Service<Request<B>> for StaticClientService<B>
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