// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::cip::{MessageRouterReply, MessageRouterRequest};
use crate::codec::Encodable;
use crate::Result;
use bytes::Bytes;

#[async_trait::async_trait(?Send)]
pub trait MessageRouter {
    /// send Heartbeat message to keep underline transport alive
    async fn heartbeat(&mut self) -> Result<()>;

    /// send message router request
    async fn send<P, D>(
        &mut self,
        mr: MessageRouterRequest<P, D>,
    ) -> Result<MessageRouterReply<Bytes>>
    where
        P: Encodable,
        D: Encodable;

    /// close underline transport
    async fn close(&mut self) -> Result<()>;

    /// is underline transport closed?
    fn closed(&self) -> bool;
}
