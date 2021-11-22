// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::*;
use crate::{
    cip::epath::EPATH_CONNECTION_MANAGER,
    codec::Encodable,
    eip::context::EipContext,
    service::reply::{ConnectedSendReply, UnconnectedSendReply},
    Result,
};
use tokio::io::{AsyncRead, AsyncWrite};

#[async_trait::async_trait(?Send)]
impl<T> Service for EipContext<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    /// context is open?
    fn is_open(&mut self) -> bool {
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

    /// send message router request without CIP connection
    #[inline]
    async fn unconnected_send<P, D>(
        &mut self,
        request: UnconnectedSend<P, D>,
    ) -> Result<MessageRouterReply<Bytes>>
    where
        P: Encodable,
        D: Encodable,
    {
        let UnconnectedSend {
            priority_ticks,
            timeout_ticks,
            path: route_path,
            data: mr_data,
        } = request;
        let mr_data_len = mr_data.bytes_count();
        let path_len = route_path.bytes_count();

        assert!(mr_data_len <= u16::MAX as usize);
        debug_assert!(path_len % 2 == 0);
        assert!(path_len <= u8::MAX as usize);

        let unconnected_send: MessageRouterRequest<&[u8], _> = MessageRouterRequest {
            service_code: SERVICE_UNCONNECTED_SEND,
            path: EPATH_CONNECTION_MANAGER,
            data: LazyEncode {
                f: move |buf: &mut BytesMut| {
                    buf.put_u8(priority_ticks);
                    buf.put_u8(timeout_ticks);

                    buf.put_u16_le(mr_data_len as u16); // size of MR
                    mr_data.encode(buf)?;
                    if mr_data_len % 2 == 1 {
                        buf.put_u8(0); // padded 0
                    }

                    buf.put_u8(path_len as u8 / 2); // path size in words
                    buf.put_u8(0); // reserved
                    route_path.encode(buf)?; // padded epath
                    Ok(())
                },
                bytes_count: 4 + mr_data_len + mr_data_len % 2 + 2 + path_len,
            },
        };

        let res: UnconnectedSendReply<Bytes> = self.send_rrdata(unconnected_send).await?;
        Ok(res.0)
    }

    /// send message router request with CIP explicit messaging connection
    #[inline]
    async fn connected_send<D>(
        &mut self,
        connection_id: u32,
        sequence_number: u16,
        request: D,
    ) -> Result<MessageRouterReply<Bytes>>
    where
        D: Encodable,
    {
        let res: ConnectedSendReply<Bytes> = self
            .send_unit_data(connection_id, sequence_number, request)
            .await?;
        Ok(res.0)
    }

    /// open CIP connection
    #[inline]
    async fn forward_open<P>(&mut self, request: Options<P>) -> Result<ForwardOpenReply>
    where
        P: Encodable,
    {
        let mr: MessageRouterRequest<&[u8], _> = MessageRouterRequest {
            service_code: SERVICE_FORWARD_OPEN,
            path: EPATH_CONNECTION_MANAGER,
            data: request,
        };
        let res: ForwardOpenReply = self.send_rrdata(mr).await?;
        Ok(res)
    }

    /// close CIP connection
    #[inline]
    async fn forward_close<P>(
        &mut self,
        request: ForwardCloseRequest<P>,
    ) -> Result<ForwardCloseReply>
    where
        P: Encodable,
    {
        let mr: MessageRouterRequest<&[u8], _> = MessageRouterRequest {
            service_code: SERVICE_FORWARD_CLOSE,
            path: EPATH_CONNECTION_MANAGER,
            data: request,
        };
        let res: ForwardCloseReply = self.send_rrdata(mr).await?;
        Ok(res)
    }
}
