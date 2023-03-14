// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::*;
use futures_util::{future::BoxFuture, stream, FutureExt, Stream, StreamExt};
use rseip_cip::identity::IdentityObject;
use rseip_eip::Discovery;
pub use rseip_eip::{consts::*, EipContext};
use rseip_rt::{CurrentRuntime, Runtime};
use std::{
    borrow::Cow,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

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

#[derive(Debug)]
pub struct EipDiscovery {
    listen_addr: SocketAddrV4,
    broadcast_addr: SocketAddrV4,
    times: Option<usize>,
    interval: Duration,
}

impl EipDiscovery {
    pub fn new(listen_addr: Ipv4Addr) -> Self {
        Self {
            listen_addr: SocketAddrV4::new(listen_addr, 0),
            broadcast_addr: SocketAddrV4::new(Ipv4Addr::BROADCAST, EIP_DEFAULT_PORT),
            times: Some(1),
            interval: Duration::from_secs(3),
        }
    }

    pub fn broadcast(mut self, ip: Ipv4Addr) -> Self {
        self.broadcast_addr = SocketAddrV4::new(ip, EIP_DEFAULT_PORT);
        self
    }

    pub fn broadcast_with_port(mut self, ip: Ipv4Addr, port: u16) -> Self {
        self.broadcast_addr = SocketAddrV4::new(ip, port);
        self
    }

    pub fn repeat(mut self, times: usize) -> Self {
        self.times = Some(times);
        self
    }

    pub fn forever(mut self) -> Self {
        self.times = None;
        self
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub async fn run(
        self,
    ) -> io::Result<impl Stream<Item = (SocketAddr, IdentityObject<'static>)>> {
        type ADiscovery =
            Discovery<<CurrentRuntime as Runtime>::UdpSocket, IdentityObject<'static>, ClientError>;
        let interval = self.interval;
        let mut times = self.times;
        let rng = std::iter::from_fn(move || match times {
            Some(0) => None,
            Some(ref mut v) => {
                *v -= 1;
                Some(())
            }
            None => Some(()),
        });

        let discover = ADiscovery::new(self.listen_addr.into(), self.broadcast_addr.into())?;
        let (mut tx, rx) = discover.split();
        let fut_send = Box::pin(async move {
            for _ in rng {
                if tx.send().await.is_err() {
                    break;
                }
                CurrentRuntime::sleep(interval).await;
            }
        })
        .fuse();
        let stream = stream::unfold((rx.fuse(), fut_send), |mut state| async move {
            futures_util::select! {
                _ = &mut state.1 => return None,
                res = state.0.next() => {
                    match res {
                        Some(Ok(v)) => return Some((v, state)),
                        Some(Err(e)) => {
                            dbg!(e);
                            return None;
                        }
                        _ => return None
                    }
                },
            }
        });
        Ok(stream)
    }
}
