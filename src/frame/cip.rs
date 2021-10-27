pub mod connection;
pub mod epath;
mod status;
pub mod types;

use crate::{codec::Encodable, objects::socket::SocketAddr};
use bytes::Bytes;
pub use epath::{EPath, PortSegment, Segment};
pub use status::Status;

/// Message request
#[derive(Debug, Default, PartialEq, Eq)]
pub struct MessageRouterRequest<P, D> {
    /// service request code
    pub service_code: u8,
    /// service request path
    pub path: P,
    /// service request data
    pub data: D,
}

impl<P, D> MessageRouterRequest<P, D>
where
    P: Encodable,
    D: Encodable,
{
    #[inline(always)]
    pub fn new(service_code: u8, path: P, data: D) -> Self {
        Self {
            service_code,
            path,
            data,
        }
    }

    #[inline(always)]
    pub fn set_service_code(mut self, service_code: u8) -> Self {
        self.service_code = service_code;
        self
    }
    #[inline(always)]
    pub fn set_path(mut self, path: P) -> Self {
        self.path = path;
        self
    }
    #[inline(always)]
    pub fn set_data(mut self, data: D) -> Self {
        self.data = data;
        self
    }
}

#[derive(Debug)]
pub struct MessageRouterReply<D> {
    pub reply_service: u8,
    pub status: Status,
    pub data: D,
}

impl<D> MessageRouterReply<D> {
    #[inline(always)]
    pub fn new(reply_service: u8, status: Status, data: D) -> Self {
        Self {
            reply_service,
            status,
            data,
        }
    }

    #[inline(always)]
    pub fn set_reply_service(mut self, reply_service: u8) -> Self {
        self.reply_service = reply_service;
        self
    }

    #[inline(always)]
    pub fn set_status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    #[inline(always)]
    pub fn set_data(mut self, data: D) -> Self {
        self.data = data;
        self
    }
}

pub enum AddressItem {
    /// type_id = 0; for unconnected message
    Null,
    /// type id = 0xA1
    Connected { connection_id: u32 },
    /// type id = 0x8002
    Sequenced {
        connection_id: u32,
        sequence_number: u32,
    },
}

pub enum DataItem {
    /// type id = 0xB2,
    /// data for Message Router request/reply
    Unconnected(Option<Bytes>),
    /// type id = 0xB1
    /// data for connected packet
    Connected(Option<Bytes>),
    SockAddr(SocketType, SocketAddr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    /// type id = 0x8000
    ToTarget,
    /// type id = 0x8001
    ToOriginator,
}

impl SocketType {
    /// CIP type id
    #[inline(always)]
    pub fn type_id(&self) -> u16 {
        match self {
            Self::ToTarget => 0x8000,
            Self::ToOriginator => 0x8001,
        }
    }
}

#[derive(Debug)]
pub struct UnconnectedSend<P, D> {
    pub timeout: u16,
    pub priority_ticks: u8,
    pub timeout_ticks: u8,
    /// connection path
    pub path: P,
    /// request data
    pub data: D,
}

impl<P, D> UnconnectedSend<P, D> {
    pub fn new(path: P, data: D) -> Self {
        Self {
            timeout: 0,
            priority_ticks: 0x03,
            timeout_ticks: 0xFA,
            path,
            data,
        }
    }
}

#[derive(Debug)]
pub struct UnconnectedSendReply<D>(pub MessageRouterReply<D>);

#[derive(Debug, Default)]
pub struct ConnectedSend<D> {
    pub session_handle: u32,
    pub connection_id: u32,
    pub sequence_number: Option<u32>,
    /// message router request
    pub data: D,
}

impl<D> ConnectedSend<D> {
    pub fn new(data: D) -> Self {
        Self {
            session_handle: 0,
            connection_id: 0,
            sequence_number: None,
            data,
        }
    }
}
