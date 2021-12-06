// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

//#![warn(missing_docs)]
#![allow(clippy::match_like_matches_macro)]

pub mod codec;
pub mod connection;
mod cpf;
pub mod epath;
pub mod error;
pub mod identity;
mod list_service;
pub mod message;
mod revision;
pub mod service;
pub mod socket;
mod status;

use core::result::Result as StdResult;
pub use cpf::*;
pub use list_service::ListServiceItem;
pub use message::*;
pub use revision::Revision;
pub use rseip_core::cip::{CommonPacket, CommonPacketItem};
pub use status::Status;
