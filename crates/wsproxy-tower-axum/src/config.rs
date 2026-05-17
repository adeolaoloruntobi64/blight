use std::{ops::Deref, sync::Arc};

use common::dns::DnsResolver;

#[derive(Clone)]
pub struct ArcedWsProxyServerInfo {
    pub inner: Arc<WsProxyServerInfo>
}

impl Deref for ArcedWsProxyServerInfo {
    type Target = Arc<WsProxyServerInfo>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct WsProxyServerInfo {
    pub allow_non_global_ip: bool,
    pub allow_non_internet_ports: bool,
    // https://github.com/MercuryWorkshop/epoxy-tls/blob/8bc68dbd71742d3c083170162f030770b1cda215/server/src/config.rs#L91
    pub allow_non_standard_udp: bool,
    pub max_message_size: usize
}

pub struct WsProxyServerConfig {
    pub info: WsProxyServerInfo,
    pub dns: Arc<DnsResolver>
}