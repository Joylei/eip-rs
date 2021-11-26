// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

//#![warn(missing_docs)]

mod codec;
mod command;
pub mod consts;
pub(crate) mod context;
mod discover;
pub mod encapsulation;
mod error;
mod framed;

pub use codec::Frame;
pub use context::EipContext;
pub(crate) use core::result::Result as StdResult;
pub use discover::EipDiscovery;
pub use encapsulation::{EncapsulationHeader, EncapsulationPacket};
pub use error::ErrorStatus;
pub use rseip_core::cip::{CommonPacket, CommonPacketItem};
pub type Error = rseip_core::Error<ErrorStatus>;
pub type Result<T> = StdResult<T, Error>;
