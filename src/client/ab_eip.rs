// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod service;
mod tag_value;

use super::*;
use futures_util::future::BoxFuture;
pub use rseip_eip::{EipContext, EipDiscovery};
pub use service::*;
use std::net::SocketAddrV4;
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
