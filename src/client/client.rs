use crate::{
    codec::{ClientCodec, Encodable},
    consts::ENIP_DEFAULT_PORT,
    frame::{
        cip::{
            EPath, MessageRouterReply, MessageRouterRequest, UnconnectedSend, UnconnectedSendReply,
        },
        command,
        command_reply::RegisterSessionReply,
    },
    Result,
};
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use std::{convert::TryInto, future::Future, io};
use tokio::net::{lookup_host, TcpSocket, TcpStream};
use tokio_util::codec::Framed;

/// thread-safety: not thread-safe
#[derive(Debug)]
pub struct Client {
    pub(crate) state: Option<State>,
}

impl Client {
    /// open session
    #[inline(always)]
    pub fn connect<S: AsRef<str>>(host: S) -> impl Future<Output = Result<Self>> {
        Self::open_with_port(host, ENIP_DEFAULT_PORT)
    }

    #[inline]
    pub async fn open_with_port<S: AsRef<str>>(host: S, port: u16) -> Result<Self> {
        let addr = match lookup_host(format!("{}:{}", host.as_ref(), port))
            .await?
            .next()
        {
            Some(addr) => addr,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::AddrNotAvailable,
                    format!("failed to resolve host: {}", host.as_ref()),
                )
                .into())
            }
        };
        //let ipv4: Ipv4Addr = host.as_ref().parse()?;
        //let addr = SocketAddrV4::new(ipv4, port);
        let socket = TcpSocket::new_v4()?;
        let stream = socket.connect(addr.into()).await?;
        let mut service = Framed::new(stream, ClientCodec::default());

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
        Ok(Self {
            state: Some(State {
                session_handle,
                service,
            }),
        })
    }

    /// session handle
    #[inline(always)]
    pub fn session_handle(&self) -> Option<u32> {
        self.state.as_ref().map(|v| v.session_handle)
    }

    #[inline]
    pub async fn unconnected_send<P, D>(
        &mut self,
        mr: MessageRouterRequest<P, D>,
        path: EPath,
    ) -> Result<MessageRouterReply<Bytes>>
    where
        P: Encodable + Send,
        D: Encodable + Send,
    {
        let state = match self.state {
            Some(ref mut state) => state,
            None => return Err(io::Error::new(io::ErrorKind::Other, "session closed").into()),
        };
        let mut ucmm = UnconnectedSend::new(path, mr);
        ucmm.session_handle = state.session_handle;
        state.service.send(ucmm).await?;
        match state.service.next().await {
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

    /// close current session
    pub async fn close(&mut self) -> Result<()> {
        match self.state {
            Some(ref mut state) => {
                let req = command::UnRegisterSession {
                    session_handle: state.session_handle,
                };
                // TODO: handle error when socket was closed
                state.service.send(req).await?;
                self.state = None;
            }
            None => {}
        }
        Ok(())
    }

    /// is session closed?
    pub fn closed(&self) -> bool {
        self.state.is_none()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        // TODO: a graceful way to close session
        //self.close()
    }
}

#[derive(Debug)]
pub(crate) struct State {
    pub(crate) session_handle: u32,
    pub(crate) service: Framed<TcpStream, ClientCodec>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::frame::cip::Segment;
    use crate::test::block_on;
    use byteorder::ByteOrder;
    use byteorder::LittleEndian;
    use bytes::BufMut;

    use crate::frame::cip::PortSegment;

    #[test]
    fn ab_read_tag() {
        block_on(async {
            let connection_path = EPath::from(vec![Segment::Port(PortSegment::default())]);
            let mut client = Client::connect("192.168.0.83").await?;
            let mr_request = MessageRouterRequest::new(
                0x4c,
                EPath::from(vec![Segment::Symbol("test_car1_x".to_owned())]),
                ElementCount(1),
            );
            let resp = client.unconnected_send(mr_request, connection_path).await?;
            assert_eq!(resp.reply_service, 0xCC); // read tag service reply
            assert_eq!(LittleEndian::read_u16(&resp.data[0..2]), 0xC4); // DINT
            client.close().await?;
            Ok(())
        });
    }

    struct ElementCount(u16);

    impl Encodable for ElementCount {
        fn encode(self, dst: &mut bytes::BytesMut) -> Result<()> {
            dst.put_u16_le(self.0);
            Ok(())
        }
        fn bytes_count(&self) -> usize {
            2
        }
    }
}
