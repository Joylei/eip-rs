// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

//! CIP services

mod common_services;
mod heartbeat;
mod message_service;
pub mod request;

use crate::*;
#[doc(inline)]
pub use common_services::CommonServices;
#[doc(inline)]
pub use heartbeat::Heartbeat;
#[doc(inline)]
pub use message_service::MessageService;
use rseip_core::codec::{Decode, Encode};

pub const SERVICE_UNCONNECTED_SEND: u8 = 0x52;
pub const SERVICE_FORWARD_OPEN: u8 = 0x54;
pub const SERVICE_LARGE_FORWARD_OPEN: u8 = 0x5B;
pub const SERVICE_FORWARD_CLOSE: u8 = 0x4E;

/// send message request and extract the data from message reply
#[doc(hidden)]
#[inline]
pub async fn send_and_extract<'de, S, P, D, R>(
    service: &mut S,
    service_code: u8,
    path: P,
    data: D,
) -> Result<R, S::Error>
where
    S: MessageService + ?Sized,
    P: Encode + Send + Sync,
    D: Encode + Send + Sync,
    R: Decode<'de> + 'static,
{
    let mr = MessageRequest {
        service_code,
        path,
        data,
    };
    let reply: MessageReply<R> = service.send(mr).await?;
    reply.expect_service::<S::Error>(service_code + REPLY_MASK)?;
    Ok(reply.data)
}
