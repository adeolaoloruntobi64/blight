use axum::extract::FromRef;

use crate::{config::ArcedWsProxyServerInfo, dns::WsProxyDnsResolverWrapper};

#[derive(Clone)]
pub struct AppState {
    pub arcedinfo: ArcedWsProxyServerInfo,
    pub resolver: WsProxyDnsResolverWrapper,
}

impl AppState {
    pub fn new(arcedinfo: ArcedWsProxyServerInfo, resolver: WsProxyDnsResolverWrapper) -> Self {
        Self { arcedinfo, resolver }
    }
}

impl FromRef<AppState> for ArcedWsProxyServerInfo {
    fn from_ref(input: &AppState) -> Self {
        input.arcedinfo.clone()
    }
}

impl FromRef<AppState> for WsProxyDnsResolverWrapper {
    fn from_ref(input: &AppState) -> Self {
        input.resolver.clone()
    }
}