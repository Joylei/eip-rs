// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{codec::Encodable, Error, MessageReply, MessageRequest, StdResult};
use bytes::Bytes;

#[async_trait::async_trait(?Send)]
pub trait MessageService {
    type Error: From<Error>;
    /// send message router request
    async fn send<P, D>(
        &mut self,
        mr: MessageRequest<P, D>,
    ) -> StdResult<MessageReply<Bytes>, Self::Error>
    where
        P: Encodable,
        D: Encodable;

    /// close underline transport
    async fn close(&mut self) -> StdResult<(), Self::Error>;

    /// is underline transport closed?
    fn closed(&self) -> bool;
}
