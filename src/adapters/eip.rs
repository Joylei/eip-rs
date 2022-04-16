// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::*;
use crate::{cip::epath::EPATH_CONNECTION_MANAGER, cip::service::*, ClientError, Result};
use rseip_cip::codec::decode::message_reply;
use rseip_core::codec::{Decode, Encode};
use rseip_eip::EipContext;
use tokio::io::{AsyncRead, AsyncWrite};

#[async_trait::async_trait]
impl<T> Service for EipContext<T, ClientError>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    /// context is open?
    fn is_open(&self) -> bool {
        self.session_handle().is_some()
    }

    /// open context
    async fn open(&mut self) -> Result<()> {
        if !self.has_session() {
            self.register_session().await?;
        }
        Ok(())
    }

    /// close context
    async fn close(&mut self) -> Result<()> {
        if self.has_session() {
            self.unregister_session().await?;
        }
        Ok(())
    }

    /// send Heartbeat message to keep underline transport alive
    #[inline]
    async fn heartbeat(&mut self) -> Result<()> {
        self.nop(()).await?;
        Ok(())
    }

    /// send CIP message request without CIP connection
    #[inline]
    async fn unconnected_send<'de, CP, P, D, R>(
        &mut self,
        request: UnconnectedSend<CP, MessageRequest<P, D>>,
    ) -> Result<R>
    where
        CP: Encode + Send + Sync,
        P: Encode + Send + Sync,
        D: Encode + Send + Sync,
        R: MessageReplyInterface + Decode<'de> + 'static,
    {
        let service_code = request.data.service_code;

        let unconnected_send: MessageRequest<&[u8], _> = MessageRequest {
            service_code: SERVICE_UNCONNECTED_SEND,
            path: EPATH_CONNECTION_MANAGER,
            data: request,
        };

        let cpf = self.send_rrdata(unconnected_send).await?;
        let reply: R = message_reply::decode_unconnected_send(cpf)?;
        reply.expect_service::<ClientError>(service_code + 0x80)?;
        Ok(reply)
    }

    /// send CIP message request with CIP explicit messaging connection
    #[inline]
    async fn connected_send<'de, P, D, R>(
        &mut self,
        connection_id: u32,
        sequence_number: u16,
        request: MessageRequest<P, D>,
    ) -> Result<R>
    where
        P: Encode + Send + Sync,
        D: Encode + Send + Sync,
        R: MessageReplyInterface + Decode<'de> + 'static,
    {
        let service_code = request.service_code;
        let cpf = self
            .send_unit_data(connection_id, sequence_number, request)
            .await?;

        let (seq_reply, reply): (_, R) = message_reply::decode_connected_send(cpf)?;
        debug_assert_eq!(sequence_number, seq_reply);
        reply.expect_service::<ClientError>(service_code + 0x80)?;
        Ok(reply)
    }

    /// open CIP connection
    #[inline]
    async fn forward_open<P>(&mut self, request: OpenOptions<P>) -> Result<ForwardOpenReply>
    where
        P: Encode + Send + Sync,
    {
        let req: MessageRequest<&[u8], _> = MessageRequest {
            service_code: SERVICE_FORWARD_OPEN,
            path: EPATH_CONNECTION_MANAGER,
            data: request,
        };

        let cpf = self.send_rrdata(req).await?;
        let reply: ForwardOpenReply = message_reply::decode_unconnected_send(cpf)?;
        Ok(reply)
    }

    /// close CIP connection
    #[inline]
    async fn forward_close<P>(
        &mut self,
        request: ForwardCloseRequest<P>,
    ) -> Result<ForwardCloseReply>
    where
        P: Encode + Send + Sync,
    {
        let req: MessageRequest<&[u8], _> = MessageRequest {
            service_code: SERVICE_FORWARD_CLOSE,
            path: EPATH_CONNECTION_MANAGER,
            data: request,
        };

        let cpf = self.send_rrdata(req).await?;
        let reply: ForwardCloseReply = message_reply::decode_unconnected_send(cpf)?;
        Ok(reply)
    }
}
