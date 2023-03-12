// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::*;
use futures_util::future::BoxFuture;
pub use rseip_eip::{consts::*, EipContext};
use rseip_rt::{CurrentRuntime, Runtime};
use std::{borrow::Cow, net::SocketAddrV4};

//pub type EipDiscovery = super::EipDiscovery<ClientError>;

/// Generic EIP Client
pub type EipClient = Client<EipDriver>;

/// Generic EIP Connection
pub type EipConnection = Connection<EipDriver>;

/// Generic EIP driver
pub struct EipDriver;

impl Driver for EipDriver {
    type Endpoint = SocketAddrV4;
    type Service = EipContext<<CurrentRuntime as Runtime>::Transport, ClientError>;

    fn build_service(addr: Self::Endpoint) -> BoxFuture<'static, Result<Self::Service>> {
        let fut = async move {
            let stream = <CurrentRuntime as Runtime>::Transport::connect(addr).await?;
            let service = EipContext::new(stream);
            Ok(service)
        };
        Box::pin(fut)
    }
}

impl<B: Driver<Endpoint = SocketAddrV4>> Client<B> {
    /// create connection from specified host, with default port if port not specified
    pub async fn new_host_lookup(host: impl AsRef<str>) -> io::Result<Self> {
        let addr = resolve_host(host).await?;
        Ok(Self::new(addr))
    }
}

impl<B: Driver<Endpoint = SocketAddrV4>> Connection<B> {
    /// create connection from specified host, with default port if port not specified
    pub async fn new_host_lookup(host: impl AsRef<str>, options: OpenOptions) -> io::Result<Self> {
        let addr = resolve_host(host).await?;
        Ok(Self::new(addr, options))
    }
}

async fn resolve_host(host: impl AsRef<str>) -> io::Result<SocketAddrV4> {
    let host: Cow<_> = {
        let host = host.as_ref();
        if host.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid host"));
        }
        if !host.contains(':') {
            Cow::Owned(format!("{}:{}", host, EIP_DEFAULT_PORT))
        } else {
            host.into()
        }
    };
    CurrentRuntime::lookup_host(host.to_string()).await
}
