use crate::{
    codec::{encode::LazyEncode, ClientCodec, Encodable},
    frame::{
        cip::{
            connection::{ConnectionParameters, ForwardCloseRequest, ForwardOpenReply},
            epath::EPATH_CONNECTION_MANAGER,
            MessageRouterReply, MessageRouterRequest, UnconnectedSend, UnconnectedSendReply,
        },
        command::{self, SendRRData},
        command_reply::RegisterSessionReply,
    },
    Result,
};
use bytes::{BufMut, Bytes, BytesMut};
use futures_util::{SinkExt, StreamExt};
use std::{convert::TryInto, io};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;

/// CIP context
pub trait Context {
    type Stream: AsyncRead + AsyncWrite;

    fn from_stream(stream: Self::Stream) -> Result<Self>
    where
        Self: Sized;

    /// session handle
    fn session_handle(&self) -> Option<u32>;

    fn session_handle_mut(&mut self) -> &mut Option<u32>;

    fn framed(&self) -> &Framed<Self::Stream, ClientCodec>;

    fn framed_mut(&mut self) -> &mut Framed<Self::Stream, ClientCodec>;
}

/// CIP service over TCP
#[async_trait::async_trait(?Send)]
pub trait TcpService: Context {
    #[inline]
    async fn register_session(&mut self) -> Result<u32>
    where
        Self::Stream: Unpin,
    {
        let service = self.framed_mut();
        service.send(command::RegisterSession).await?;
        let session_handle = match service.next().await {
            Some(resp) => {
                let RegisterSessionReply {
                    session_handle,
                    protocol_version,
                } = resp?.try_into()?;
                debug_assert_eq!(protocol_version, 1);
                session_handle
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "RegisterSession: connection lost",
                )
                .into())
            }
        };
        Ok(session_handle)
    }

    #[inline]
    async fn unregister_session(&mut self) -> Result<()>
    where
        Self::Stream: Unpin,
    {
        if let Some(session_handle) = self.session_handle() {
            let service = self.framed_mut();
            service
                .send(command::UnRegisterSession { session_handle })
                .await?;
        }
        Ok(())
    }

    #[inline]
    async fn unconnected_send<P, D>(
        &mut self,
        request: UnconnectedSend<P, D>,
    ) -> Result<MessageRouterReply<Bytes>>
    where
        Self::Stream: Unpin,
        P: Encodable,
        D: Encodable,
    {
        const SERVICE_UNCONNECTED_SEND: u8 = 0x52;
        let session_handle = match self.session_handle() {
            Some(h) => h,
            None => return Err(io::Error::new(io::ErrorKind::Other, "CIP session required").into()),
        };
        let service = self.framed_mut();
        let UnconnectedSend {
            priority_ticks,
            timeout_ticks,
            timeout,
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
        let command = SendRRData {
            session_handle,
            timeout,
            data: unconnected_send,
        };
        service.send(command).await?;

        match service.next().await {
            Some(resp) => {
                let mr_reply: UnconnectedSendReply<Bytes> = resp?.try_into()?;
                Ok(mr_reply.0)
            }
            None => Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "UnconnectedSend: connection lost",
            )
            .into()),
        }
    }

    #[inline]
    async fn forward_open<P>(
        &mut self,
        parameters: ConnectionParameters<P>,
    ) -> Result<ForwardOpenReply>
    where
        Self::Stream: Unpin,
        P: Encodable,
    {
        const SERVICE_FORWARD_OPEN: u8 = 0x54;
        let session_handle = match self.session_handle() {
            Some(h) => h,
            None => return Err(io::Error::new(io::ErrorKind::Other, "CIP session required").into()),
        };
        let service = self.framed_mut();
        let mr: MessageRouterRequest<&[u8], _> = MessageRouterRequest {
            service_code: SERVICE_FORWARD_OPEN,
            path: EPATH_CONNECTION_MANAGER,
            data: parameters,
        };
        let command = SendRRData {
            session_handle,
            timeout: 0,
            data: mr,
        };
        service.send(command).await?;
        match service.next().await {
            Some(resp) => {
                let mr_reply: ForwardOpenReply = resp?.try_into()?;
                Ok(mr_reply)
            }
            None => Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "ForwardOpen: connection lost",
            )
            .into()),
        }
    }

    #[inline]
    async fn forward_close<P>(&mut self, request: ForwardCloseRequest<P>) -> Result<()>
    where
        Self::Stream: Unpin,
        P: Encodable,
    {
        const SERVICE_FORWARD_CLOSE: u8 = 0x4E;
        let session_handle = match self.session_handle() {
            Some(h) => h,
            None => return Err(io::Error::new(io::ErrorKind::Other, "CIP session required").into()),
        };
        let service = self.framed_mut();
        let mr: MessageRouterRequest<&[u8], _> = MessageRouterRequest {
            service_code: SERVICE_FORWARD_CLOSE,
            path: EPATH_CONNECTION_MANAGER,
            data: request,
        };
        let command = SendRRData {
            session_handle,
            timeout: 0,
            data: mr,
        };
        service.send(command).await?;
        match service.next().await {
            Some(resp) => {
                let _mr_reply: UnconnectedSendReply<Bytes> = resp?.try_into()?;
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "ForwardClose: connection lost",
            )
            .into()),
        }
    }
}
