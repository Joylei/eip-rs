// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod eip;

use crate::{
    cip::{
        connection::{ForwardCloseReply, ForwardCloseRequest, ForwardOpenReply, OpenOptions},
        service::request::UnconnectedSend,
        MessageRequest,
    },
    Result,
};
use rseip_cip::MessageReplyInterface;
use rseip_core::codec::{Decode, Encode};

/// abstraction for basic CIP services;
/// different transport protocols derive this trait, eg EIP, DF1
#[async_trait::async_trait]
pub trait Service: Send + Sync {
    /// context is open?
    fn is_open(&self) -> bool;

    /// open context, eg register session for EIP
    async fn open(&mut self) -> Result<()>;

    /// close context, eg unregister session for EIP
    async fn close(&mut self) -> Result<()>;

    /// send Nop to keep underline transport alive
    async fn heartbeat(&mut self) -> Result<()> {
        Ok(())
    }

    /// unconnected send
    async fn unconnected_send<'de, CP, P, D, R>(
        &mut self,
        request: UnconnectedSend<CP, MessageRequest<P, D>>,
    ) -> Result<R>
    where
        CP: Encode + Send + Sync,
        P: Encode + Send + Sync,
        D: Encode + Send + Sync,
        R: MessageReplyInterface + Decode<'de> + 'static;

    /// connected send
    async fn connected_send<'de, P, D, R>(
        &mut self,
        connection_id: u32,
        sequence_number: u16,
        request: MessageRequest<P, D>,
    ) -> Result<R>
    where
        P: Encode + Send + Sync,
        D: Encode + Send + Sync,
        R: MessageReplyInterface + Decode<'de> + 'static;

    /// forward open
    async fn forward_open<P>(&mut self, request: OpenOptions<P>) -> Result<ForwardOpenReply>
    where
        P: Encode + Send + Sync;

    /// forward close
    async fn forward_close<P>(
        &mut self,
        request: ForwardCloseRequest<P>,
    ) -> Result<ForwardCloseReply>
    where
        P: Encode + Send + Sync;
}
