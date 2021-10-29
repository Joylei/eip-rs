// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2020-2021, Joylei <leingliu@gmail.com>
// License: MIT

/// default port for EtherNet/IP over TCP/IP
pub const EIP_DEFAULT_PORT: u16 = 0xAF12;
/// default port for EtherNet/IP over TCP/IP class 0 and class 1
pub const EIP_DEFAULT_UDP_PORT: u16 = 0x08AE;

pub const ENCAPSULATION_HEADER_LEN: usize = 24;
pub(crate) const ENCAPSULATION_DATA_MAX_LEN: usize = u16::MAX as usize - ENCAPSULATION_HEADER_LEN;
