// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::{socket::SocketAddr, Revision};
use std::borrow::Cow;

/// Identity Object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityObject<'a> {
    /// encapsulation protocol version supported
    pub protocol_version: u16,
    /// socket addr
    pub socket_addr: SocketAddr,
    /// device manufacturers vendor id
    pub vendor_id: u16,
    /// device type of product
    pub device_type: u16,
    /// product code
    pub product_code: u16,
    /// device revision
    pub revision: Revision,
    /// current status of device
    pub status: u16,
    /// serial number of device
    pub serial_number: u32,
    //pub product_name_len: u8,
    /// short string
    pub product_name: Cow<'a, str>,
    /// current state of device
    pub state: u8,
}
