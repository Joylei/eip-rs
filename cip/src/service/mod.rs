// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

//! CIP services

mod common_services;
mod heartbeat;
mod message_service;
pub mod request;

#[doc(inline)]
pub use common_services::CommonServices;
#[doc(inline)]
pub use heartbeat::Heartbeat;
#[doc(inline)]
pub use message_service::MessageService;

pub const SERVICE_UNCONNECTED_SEND: u8 = 0x52;
pub const SERVICE_FORWARD_OPEN: u8 = 0x54;
pub const SERVICE_LARGE_FORWARD_OPEN: u8 = 0x5B;
pub const SERVICE_FORWARD_CLOSE: u8 = 0x4E;
