use std::{ops::Deref, sync::Arc, time::Duration};

use common::dns::DnsResolver;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use crate::structs::BareServerVersion;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainerInfo {
    pub email: String,
    pub website: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub description: String,
    pub email: String,
    pub website: String,
    pub repository: String,
    pub version: String,
}


#[derive(Debug, Clone)]
pub struct BareServerInfo {
    pub maintainer: MaintainerInfo,
    pub project: ProjectInfo,
    pub ws_cache_ttl: Duration,
    pub extra_meta: bool,
    pub block_non_global_ips: bool,
    pub supported_versions: Vec<BareServerVersion>
}

#[derive(Debug, Clone)]
pub struct ArcedBareServerInfo {
    pub inner: Arc<BareServerInfo>
}

impl Deref for ArcedBareServerInfo {
    type Target = Arc<BareServerInfo>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Clone)]
pub struct BareServerConfig {
    pub info: BareServerInfo,
    pub dns: Arc<DnsResolver>,
    pub cors: CorsLayer,
}