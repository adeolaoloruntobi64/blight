use std::{ops::Deref, sync::Arc};

use common::dns::DnsResolver;

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