use std::{ops::Deref, sync::Arc};

use common::dns::DnsResolver;

#[derive(Debug, Clone)]
pub struct WispDnsResolverWrapper {
    pub resolver: Arc<DnsResolver>
}

impl Deref for WispDnsResolverWrapper {
    type Target = Arc<DnsResolver>;

    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}