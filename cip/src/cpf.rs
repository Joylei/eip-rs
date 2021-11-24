// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::socket::{SocketAddr, SocketType};
use bytes::Bytes;

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
