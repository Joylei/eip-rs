// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Revision {
    pub major: u8,
    pub minor: u8,
}
