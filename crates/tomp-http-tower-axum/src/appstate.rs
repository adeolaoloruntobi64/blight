use axum::extract::FromRef;
use reqwest::{redirect::Policy, Client};
use moka::future::Cache;

use crate::{config::ArcedBareServerInfo, dns::TompDnsResolverWrapper, structs::WsMeta};

#[derive(Clone)]
pub struct Requestor {
    /// This server only sends http/1.1 requests for maximum compatibility with other websites
    pub http11client: Client,
}

#[derive(Clone)]
pub struct BareWSStore {
    pub wsdata: Cache<String, Option<WsMeta>>
}

#[derive(Clone)]
pub struct AppState {
    pub arcedinfo: ArcedBareServerInfo,
    pub requestor: Requestor,
    pub resolver: TompDnsResolverWrapper,
    pub wsstore: BareWSStore,
}

impl AppState {
    pub fn new(arcedinfo: ArcedBareServerInfo, resolver: TompDnsResolverWrapper) -> Self {
        // The Policy::none() is VERY VERY IMPORTANT. THE CLIENT HANDLES REDIRECTS, NOT US
        // I HAD PROBLEMS WITH GOOGLE CHANGING LANGUAGES FROM ARABIC TO ENGLISH FOR SO LONG
        // BECAUSE IT RETURED THE GOOGLE START PAGE INSTEAD OF THE 302 PAGE BRUHHHHHHHHHH
        // I SPENT A FULL DAY SOLELY ON THIS, AND PARTS OF OTHER DAYS TOO AAAAAAAAAAAAAAAA
        let client = Client::builder()
            .tls_backend_rustls()
            .http1_only()
            .http1_title_case_headers() // https://github.com/tomphttp/specifications/blob/master/BareServerV3.md#bare-request-headers
            .no_gzip()
            .no_brotli()
            .no_zstd()
            .no_deflate()
            .redirect(Policy::none())
            .dns_resolver((*resolver).clone())
            .build()
            .unwrap();
        
        // TTL is supposed to be 30s, but just like with CORS, I doubled it
        let cache = Cache::builder()
            .time_to_live(arcedinfo.inner.ws_cache_ttl)
            .build();
        
        Self {
            arcedinfo,
            requestor: Requestor { http11client: client },
            resolver,
            wsstore: BareWSStore { wsdata: cache },
        }
    }
}

impl FromRef<AppState> for ArcedBareServerInfo {
    fn from_ref(input: &AppState) -> Self {
        input.arcedinfo.clone()
    }
}

impl FromRef<AppState> for Requestor {
    fn from_ref(input: &AppState) -> Self {
        input.requestor.clone()
    }
}

impl FromRef<AppState> for TompDnsResolverWrapper {
    fn from_ref(input: &AppState) -> Self {
        input.resolver.clone()
    }
}

impl FromRef<AppState> for BareWSStore {
    fn from_ref(input: &AppState) -> Self {
        input.wsstore.clone()
    }
}