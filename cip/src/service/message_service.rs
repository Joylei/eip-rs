// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{MessageReplyInterface, MessageRequest};
use rseip_core::{
    codec::{Decode, Encode},
    Error,
};

#[async_trait::async_trait(?Send)]
pub trait MessageService {
    type Error: Error;
    /// send message request
    async fn send<'de, P, D, R>(&mut self, mr: MessageRequest<P, D>) -> Result<R, Self::Error>
    where
        P: Encode,
        D: Encode,
        R: MessageReplyInterface + Decode<'de> + 'static;

    /// close underline transport
    async fn close(&mut self) -> Result<(), Self::Error>;

    /// is underline transport closed?
    fn closed(&self) -> bool;
}

#[async_trait::async_trait(?Send)]
impl<T: MessageService + Sized> MessageService for &mut T {
    type Error = T::Error;
    #[inline]
    async fn send<'de, P, D, R>(&mut self, mr: MessageRequest<P, D>) -> Result<R, Self::Error>
    where
        P: Encode,
        D: Encode,
        R: MessageReplyInterface + Decode<'de> + 'static,
    {
        (**self).send(mr).await
    }

    #[inline]
    async fn close(&mut self) -> Result<(), Self::Error> {
        (**self).close().await
    }

    #[inline]
    fn closed(&self) -> bool {
        (**self).closed()
    }
}
