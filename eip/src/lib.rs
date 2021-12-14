// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

//#![warn(missing_docs)]
#![allow(clippy::match_like_matches_macro)]

mod codec;
mod command;
pub mod consts;
pub(crate) mod context;
mod discover;
pub mod encapsulation;
mod error;
mod framed;

pub use context::EipContext;
pub use discover::EipDiscovery;
pub use encapsulation::{EncapsulationHeader, EncapsulationPacket};
pub use rseip_core::{
    cip::{CommonPacket, CommonPacketItem},
    Error,
};
