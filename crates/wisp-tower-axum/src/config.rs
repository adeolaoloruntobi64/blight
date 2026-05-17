use std::{collections::HashMap, ops::Deref, sync::Arc};

use common::dns::DnsResolver;
use wisp_mux::extensions::cert::VerifyKey;
use crate::versions::WispServerVersion;

#[derive(Clone)]
pub struct ArcedWispServerInfo {
    pub inner: Arc<WispServerInfo>
}

impl Deref for ArcedWispServerInfo {
    type Target = Arc<WispServerInfo>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct WispServerInfo {
    pub allow_non_global_ip: bool,
    pub allow_non_internet_ports: bool,
    pub v2_allow_udp: bool,
    pub v2_use_auth: Option<HashMap<String, String>>,
    pub v2_use_motd: Option<String>,
    pub v2_use_cert: Vec<VerifyKey>,
    pub supported_versions: Vec<WispServerVersion>,
    pub max_message_size: usize
}

pub struct WispServerConfig {
    pub info: WispServerInfo,
    pub dns: Arc<DnsResolver>
}