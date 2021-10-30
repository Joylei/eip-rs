// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

pub mod identity;
pub mod service;
pub mod socket;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Revision {
    pub major: u8,
    pub minor: u8,
}
