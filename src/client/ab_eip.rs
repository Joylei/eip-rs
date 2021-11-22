// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod service;
mod tag_value;

use super::*;
use crate::eip::context::EipContext;
use futures_util::future::BoxFuture;
pub use service::*;
use std::{io, net::SocketAddrV4};
pub use tag_value::TagValue;
use tokio::net::TcpStream;

/// AB EIP Client
pub type AbEipClient = Client<AbEipDriver>;

/// AB EIP Connection
pub type AbEipConnection = Connection<AbEipDriver>;

/// AB EIP driver
pub struct AbEipDriver;

impl Driver for AbEipDriver {
    type Endpoint = SocketAddrV4;
    type Service = EipContext<TcpStream>;

    #[inline]
    fn build_service(addr: &Self::Endpoint) -> BoxFuture<Result<Self::Service>> {
        EipDriver::build_service(addr)
    }
}

impl AbEipClient {
    pub async fn new_host_lookup(host: impl AsRef<str>) -> io::Result<Self> {
        let addr = resolve_host(host).await?;
        Ok(Self::new(addr))
    }
}

impl AbEipConnection {
    pub async fn new_host_lookup(host: impl AsRef<str>, options: Options) -> io::Result<Self> {
        let addr = resolve_host(host).await?;
        Ok(Self::new(addr, options))
    }
}
