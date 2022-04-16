// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::*;
use futures_util::future::BoxFuture;
pub use rseip_eip::{consts::*, EipContext};
use std::{
    borrow::Cow,
    net::{SocketAddr, SocketAddrV4},
};
use tokio::net::{lookup_host, TcpSocket, TcpStream};

pub type EipDiscovery = rseip_eip::EipDiscovery<ClientError>;

/// Generic EIP Client
pub type EipClient = Client<EipDriver>;

/// Generic EIP Connection
pub type EipConnection = Connection<EipDriver>;

/// Generic EIP driver
pub struct EipDriver;

impl Driver for EipDriver {
    type Endpoint = SocketAddrV4;
    type Service = EipContext<TcpStream, ClientError>;

    fn build_service(addr: Self::Endpoint) -> BoxFuture<'static, Result<Self::Service>> {
        let fut = async move {
            let socket = TcpSocket::new_v4()?;
            let stream = socket.connect(addr.into()).await?;
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
    let addr = lookup_host(host.as_ref())
        .await?
        .filter_map(|item| match item {
            SocketAddr::V4(addr) => Some(addr),
            _ => None,
        })
        .next();
    if let Some(addr) = addr {
        Ok(addr)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "dns lookup failure",
        ))
    }
}
