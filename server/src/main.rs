use std::{convert::Infallible, marker::PhantomData, net::SocketAddr, sync::Arc, time::Duration};

use axum::{body::Body, extract::{OriginalUri, Request}, http::{Response, StatusCode}, Router};
use clap::Parser;
use common::dns::{DnsResolver, hickory_resolver::{TokioResolver, config::{CLOUDFLARE, LookupIpStrategy, ResolverConfig, ResolverOpts}, net::runtime::TokioRuntimeProvider}};
use config::Config;
use mimalloc::MiMalloc;
use static_client_tower_axum::{config::StaticClientConfig, StaticClientService};
use tomp_http_tower_axum::{config::{BareServerConfig, BareServerInfo, MaintainerInfo, ProjectInfo}, BareServerService, BareServerVersion};
use tokio::{net::TcpListener, runtime::Builder};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, self as ts};
use wisp_tower_axum::{config::{WispServerConfig, WispServerInfo}, versions::WispServerVersion, WispServerService};
use wsproxy_tower_axum::{config::{WsProxyServerConfig, WsProxyServerInfo}, WsProxyServerService};

mod config;
mod auth;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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

async fn server() {
    let settings = Config::parse();
    let socket = settings.socket.clone();
    let listener = TcpListener::bind(socket).await.unwrap();

    ts::registry().with(ts::fmt::layer().pretty()).with(
        tracing_subscriber::EnvFilter::from_default_env()
    ).init();

    let dns = {
        let mut options = ResolverOpts::default();
        // We don't want sending 2 requests at the same time, because
        // that wastes gb limit, although not much, it might add up idk
        options.num_concurrent_reqs = 4;
        options.cache_size = 1024;
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
                name: "Blight".into(),
                description: "Rust TOMP implementation".into(),
                email: "None".into(),
                website: "None".into(),
                repository: "I should make a github".into(),
                version: "0.1.0".into()
            },
            ws_cache_ttl: Duration::from_secs(60),
            extra_meta: settings.extra_bare_meta,
            block_non_global_ips: !settings.allow_non_global_ip,
            supported_versions: vec![BareServerVersion::V1, BareServerVersion::V2, BareServerVersion::V3]
        },
        dns: dns.clone(),
        cors: CorsLayer::permissive().max_age(Duration::from_secs(60) * 10)
    };

    let wisp_config = WispServerConfig {
        info: WispServerInfo {
            allow_non_global_ip: settings.allow_non_global_ip,
            allow_non_internet_ports: settings.allow_non_internet_ports,
            v2_allow_udp: settings.allow_udp,
            v2_use_auth: settings.auth_path.as_ref().map(|p|
                auth::parse_auth_file(p).expect("Couldn't parse auth file")
            ),
            v2_use_motd: None,
            v2_use_cert: Vec::new(),
            supported_versions: vec![WispServerVersion::V1, WispServerVersion::V2],
            max_message_size: settings.ws_max_message_size,
            buffer_size: 1024
        },
        dns: dns.clone()
    };

    let wsproxy_config = WsProxyServerConfig {
        info: WsProxyServerInfo {
            allow_non_global_ip: settings.allow_non_global_ip,
            allow_non_internet_ports: settings.allow_non_internet_ports,
            allow_non_standard_udp: settings.allow_udp,
            max_message_size: settings.ws_max_message_size
        },
        dns,
    };

    let static_client_config = StaticClientConfig {
        channel_size: 1024,
        path: settings.frontend_files_dir.clone(),
        fallback_service: Some(tower::service_fn(fallback)),
        phantomdata: PhantomData::<[u16; 1024]>
    };

    // Used to use 'nest_service('\', ...), but https://github.com/tokio-rs/axum/issues/2651
    // Also realized in Docker, it ws using multiple MB. Turns out ServeDir doesn't compress
    // (but it seems to be able to cache / return 304), so made StaticClientService
    let service = Router::new()
        .nest_service(&settings.bare_prefix, BareServerService::new(bare_config))
        .nest_service(&settings.wisp_prefix, WispServerService::new(wisp_config))
        .nest_service(&settings.wsproxy_prefix, WsProxyServerService::new(wsproxy_config))
        .fallback_service(StaticClientService::new(static_client_config))
        .into_make_service_with_connect_info::<SocketAddr>();

    tracing::info!("Server starting. Configutation:\n{settings:?}");

    axum::serve(listener, service)
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
        .await
        .unwrap();
    
    tracing::info!("Shutting down");
}

fn main() {
    Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(server())
}
