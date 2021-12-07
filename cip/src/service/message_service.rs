// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{MessageReplyInterface, MessageRequest, StdResult};
use rseip_core::{
    codec::{Decode, Encode},
    Error,
};

#[async_trait::async_trait(?Send)]
pub trait MessageService {
    type Error: Error;
    /// send message request
    async fn send<'de, P, D, R>(&mut self, mr: MessageRequest<P, D>) -> StdResult<R, Self::Error>
    where
        P: Encode,
        D: Encode,
        R: MessageReplyInterface + Decode<'de> + 'static;

    /// close underline transport
    async fn close(&mut self) -> StdResult<(), Self::Error>;

    /// is underline transport closed?
    fn closed(&self) -> bool;
}
