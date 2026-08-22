use std::{net::SocketAddr, ops::Deref, sync::Arc};

use common::dns::DnsResolver;
use tokio_websockets::{Error, resolver::Resolver};

#[derive(Debug, Clone)]
pub struct TompDnsResolverWrapper {
    pub resolver: Arc<DnsResolver>
}

impl Deref for TompDnsResolverWrapper {
    type Target = Arc<DnsResolver>;

    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}

impl Resolver for TompDnsResolverWrapper {
    async fn resolve(
        &self,
        host: &str,
        port: u16,
    ) -> Result<SocketAddr, Error> {
        self.resolver.resolve(host, port).await
    }
}