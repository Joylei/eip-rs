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
mod error;
pub mod identity;
mod list_service;
pub mod message_request;
mod revision;
pub mod service;
pub mod socket;
mod status;

use core::result::Result as StdResult;
pub use cpf::*;
pub use error::CipError;
pub use list_service::ListServiceItem;
pub use message_request::*;
pub use revision::Revision;
pub use rseip_core::cip::{CommonPacket, CommonPacketItem};
pub use status::Status;

pub type Error = rseip_core::Error<CipError>;
pub type Result<T> = StdResult<T, Error>;
