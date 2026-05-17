use axum::extract::FromRef;

use crate::{config::ArcedWispServerInfo, dns::WispDnsResolverWrapper};

#[derive(Clone)]
pub struct AppState {
    pub arcedinfo: ArcedWispServerInfo,
    pub resolver: WispDnsResolverWrapper,
}

impl AppState {
    pub fn new(arcedinfo: ArcedWispServerInfo, resolver: WispDnsResolverWrapper) -> Self {
        Self { arcedinfo, resolver }
    }
}

impl FromRef<AppState> for ArcedWispServerInfo {
    fn from_ref(input: &AppState) -> Self {
        input.arcedinfo.clone()
    }
}

impl FromRef<AppState> for WispDnsResolverWrapper {
    fn from_ref(input: &AppState) -> Self {
        input.resolver.clone()
    }
}