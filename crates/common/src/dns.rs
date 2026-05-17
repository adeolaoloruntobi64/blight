use std::{future::Future, net::{IpAddr, SocketAddr}, pin::Pin, sync::Arc, task::{Context, Poll}};

use tokio::task::JoinHandle;
use hickory_resolver::{
    TokioResolver, lookup::Lookup, lookup_ip::LookupIp, net::NetError, proto::rr::{IntoName, RecordType}
};
use reqwest::dns::{Name, Resolve, Resolving};
use hyper::service::Service;

pub use hickory_resolver;

// cloning this will still point to the same resolver
#[derive(Debug, Clone)]
pub struct DnsResolver {
    pub resolver: Arc<TokioResolver>
}

pub struct DnsResAddrs {
    inner: LookupIp,
    port: u16
}

impl Iterator for DnsResAddrs {
    type Item = SocketAddr;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.iter().next().map(|addr| SocketAddr::from((addr, self.port)))
    }
}
pub struct DnsResFuture {
    inner: JoinHandle<Result<DnsResAddrs, NetError>>
}

impl Future for DnsResFuture {
    type Output = Result<DnsResAddrs, NetError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|res| match res {
            Ok(Ok(addrs)) => Ok(addrs),
            Ok(Err(err)) => Err(err),
            Err(join_err) => {
                if join_err.is_cancelled() {
                    Err(NetError::from("Join was cancelled"))
                } else {
                    panic!("DnsResFuture background task failed: {:?}", join_err)
                }
            }
        })
    }
}

impl DnsResolver {
    pub fn new(resolver: Arc<TokioResolver>) -> Self {
        Self { resolver }
    }

    pub async fn lookup_ip(&self, name: &str) -> Result<LookupIp, NetError> {
        self.resolver.lookup_ip(name).await
    }
    
    pub async fn lookup_socket(&self, name: &str) -> Result<DnsResAddrs, NetError> {
        let ips = self.lookup_ip(name).await?;
        Ok(DnsResAddrs { inner: ips, port: 0 })
    }

    pub async fn lookup_socket_with_port(&self, name: &str, port: u16) -> Result<DnsResAddrs, NetError> {
        let ips = self.lookup_ip(name).await?;
        Ok(DnsResAddrs { inner: ips, port })
    }

    // literally just lookup
    pub async fn reverse_lookup(&self, ip: IpAddr) -> Result<Lookup, NetError> {
        self.resolver.reverse_lookup(ip).await
    }

    pub async fn lookup<T: IntoName>(&self, name: T, record_type: RecordType) -> Result<Lookup, NetError> {
        self.resolver.lookup(name, record_type).await
    }
}

impl Resolve for DnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let dns_resolver = self.clone();
        Box::pin(async move {
            let sockets = dns_resolver.lookup_socket(name.as_str()).await?;
            let boxed_ips: Box<dyn Iterator<Item = SocketAddr> + Send> = Box::new(sockets);
            Ok(boxed_ips)
        })
    }
}

impl Service<Name> for DnsResolver {
    type Response = DnsResAddrs;
    type Error = NetError;
    type Future = DnsResFuture;

    fn call(&self, req: Name) -> Self::Future {
        let dns_resolver = self.clone();
        let task = tokio::spawn(async move {
            dns_resolver
                .lookup_socket(req.as_str())
                .await
        });

        DnsResFuture { inner: task }
    }
}