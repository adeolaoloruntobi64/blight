use std::{convert::Infallible, net::SocketAddr, str::FromStr, sync::Arc, time::Duration};

use argh::{from_env, FromArgs};
use axum::{body::Body, extract::{OriginalUri, Request}, http::{Response, StatusCode}, Router};
use common::dns::{DnsResolver, hickory_resolver::{TokioResolver, config::{CLOUDFLARE, LookupIpStrategy, ResolverConfig, ResolverOpts}, net::runtime::TokioRuntimeProvider}};
use tomp_http_tower_axum::{config::{BareServerConfig, BareServerInfo, MaintainerInfo, ProjectInfo}, BareServerService, BareServerVersion};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, self as ts};

#[derive(FromArgs, Debug, Clone)]
/// Bare server init
struct BareServerSettings {
    /// the bare server directory, defaults to /
    #[argh(option, short = 'd', default = "String::from(\"/\")")]
    directory: String,
    
    /// the socket to bind to, defaults to 127.0.0.1:3000
    #[argh(
        option, short = 's', from_str_fn(socket_from_str),
        default = "SocketAddr::from_str(\"127.0.0.1:3000\").unwrap()"
    )]
    socket: SocketAddr,
}

fn socket_from_str(inp: &str) -> Result<SocketAddr, String> {
    SocketAddr::from_str(inp).map_err(|err| err.to_string())
}

async fn fallback(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let url = req.extensions()
        .get::<OriginalUri>()
        .map_or(req.uri().to_string(), |ouri| ouri.to_string());

    let body = format!("No route for {url}");
    tracing::debug!(body);
    
    return Ok(
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(body))
            .unwrap()
    );
}

#[tokio::main]
async fn main() {
    let settings = from_env::<BareServerSettings>();
    let socket = settings.socket.clone();
    let listener = TcpListener::bind(socket).await.unwrap();

    ts::registry().with(ts::fmt::layer().pretty()).with(
        tracing_subscriber::EnvFilter::from_default_env()
    ).init();

    let dns = {
        let mut options = ResolverOpts::default();
        options.cache_size = 4096;
        options.ip_strategy = LookupIpStrategy::Ipv4thenIpv6;
        let resolver = TokioResolver::builder_with_config(
            ResolverConfig::https(&CLOUDFLARE),
            TokioRuntimeProvider::new()
        ).with_options(options).build().unwrap();
        Arc::new(DnsResolver::new(Arc::new(resolver)))
    };
    
    let bare_config = BareServerConfig {
        info: BareServerInfo {
            maintainer: MaintainerInfo {
                email: "you@example.com".into(),
                website: "https://www.example.com/".into()
            },
            project: ProjectInfo {
                name: "Evade-Bare".into(),
                description: "Rust TOMP implementation".into(),
                email: "None".into(),
                website: "None".into(),
                repository: "I should make a github".into(),
                version: "0.1.0".into()
            },
            ws_cache_ttl: Duration::from_secs(60),
            extra_meta: false,
            block_non_global_ips: true,
            supported_versions: vec![BareServerVersion::V1, BareServerVersion::V2, BareServerVersion::V3]
        },
        dns,
        cors: CorsLayer::permissive().max_age(Duration::from_secs(60) * 10)
    };
    
    let service = Router::new()
        .nest_service(&settings.directory, BareServerService::new(bare_config))
        .fallback(fallback)
        .into_make_service_with_connect_info::<SocketAddr>();

    let server = axum::serve(listener, service)
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() });

    tracing::info!("Server has started. Configutation:\n{settings:?}");
    server.await.unwrap();
    return;
}
