// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

pub(crate) mod interceptor;
mod path;
mod service;
mod symbol;
pub mod template;
pub mod value;

use super::*;
use futures_util::future::BoxFuture;
pub use path::{PathError, PathParser};
use rseip_cip::Status;
pub use rseip_eip::EipContext;
pub use service::*;
use std::net::SocketAddrV4;
pub use symbol::{GetInstanceAttributeList, SymbolInstance};
pub use template::AbTemplateService;
use tokio::net::TcpStream;
pub use value::*;

pub const CLASS_SYMBOL: u16 = 0x6B;
pub const CLASS_TEMPLATE: u16 = 0x6C;

pub const SERVICE_READ_TAG: u8 = 0x4C;
pub const SERVICE_WRITE_TAG: u8 = 0x4D;
pub const SERVICE_READ_TAG_FRAGMENTED: u8 = 0x52;
pub const SERVICE_WRITE_TAG_FRAGMENTED: u8 = 0x53;
pub const SERVICE_READ_MODIFY_WRITE_TAG: u8 = 0x4E;
pub const SERVICE_TEMPLATE_READ: u8 = 0x4C;

pub type EipDiscovery = rseip_eip::EipDiscovery<ClientError>;

/// AB EIP Client
pub type AbEipClient = Client<AbEipDriver>;

/// AB EIP Connection
pub type AbEipConnection = Connection<AbEipDriver>;

/// AB EIP driver
pub struct AbEipDriver;

impl Driver for AbEipDriver {
    type Endpoint = SocketAddrV4;
    type Service = EipContext<TcpStream, ClientError>;

    #[inline]
    fn build_service(addr: Self::Endpoint) -> BoxFuture<'static, Result<Self::Service>> {
        EipDriver::build_service(addr)
    }
}

/// has more data
pub trait HasMore {
    /// true: has more data to retrieve
    fn has_more(&self) -> bool;
}

impl HasMore for Status {
    /// true: has more data to retrieve
    #[inline]
    fn has_more(&self) -> bool {
        self.general == 0x06
    }
}

impl<D> HasMore for MessageReply<D> {
    #[inline]
    fn has_more(&self) -> bool {
        self.status.has_more()
    }
}
