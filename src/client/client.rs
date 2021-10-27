use crate::{
    codec::{ClientCodec, Encodable},
    consts::EIP_DEFAULT_PORT,
    frame::cip::{EPath, MessageRouterReply, MessageRouterRequest, UnconnectedSend},
    service::client::{Context, TcpService},
    Result,
};
use bytes::Bytes;
use std::io;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{lookup_host, TcpSocket, TcpStream};
use tokio_util::codec::Framed;

/// CIP client over TCP/IP
#[derive(Debug)]
pub struct Client<S: TcpService = State<TcpStream>>(S);

impl Client {
    /// connect with default port `0xAF12`, creating a CIP session
    #[inline(always)]
    pub async fn connect<S: AsRef<str>>(host: S) -> Result<Self> {
        Self::connect_with_port(host, EIP_DEFAULT_PORT).await
    }

    /// connect with specified port, creating a CIP session
    #[inline]
    pub async fn connect_with_port<S: AsRef<str>>(host: S, port: u16) -> Result<Self> {
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
        let res = Self::new(stream).await?;
        Ok(res)
    }
}

impl<S> Client<S>
where
    S: TcpService + Sized,
    S::Stream: AsyncRead + AsyncWrite + Unpin,
{
    /// create from tcp stream
    #[inline(always)]
    pub async fn new(stream: S::Stream) -> Result<Self> {
        let mut state = S::from_stream(stream)?;
        let session_handle = state.register_session().await?;
        *state.session_handle_mut() = Some(session_handle);
        Ok(Self(state))
    }
}

impl<S> Client<S>
where
    S: TcpService,
    S::Stream: AsyncRead + AsyncWrite + Unpin,
{
    /// session handle
    #[inline(always)]
    pub fn session_handle(&self) -> Option<u32> {
        self.0.session_handle()
    }

    /// unconnected send
    #[inline]
    pub async fn send<P, D>(
        &mut self,
        mr: MessageRouterRequest<P, D>,
        path: EPath,
    ) -> Result<MessageRouterReply<Bytes>>
    where
        P: Encodable,
        D: Encodable,
    {
        let request = UnconnectedSend::new(path, mr);
        let reply = self.0.unconnected_send(request).await?;
        Ok(reply)
    }

    /// open session
    #[inline(always)]
    pub async fn open(&mut self) -> Result<()> {
        if self.closed() {
            let session_handle = self.0.register_session().await?;
            *self.0.session_handle_mut() = Some(session_handle);
        }
        Ok(())
    }

    /// close current session
    #[inline(always)]
    pub async fn close(&mut self) -> Result<()> {
        self.0.unregister_session().await?;
        *self.0.session_handle_mut() = None;
        Ok(())
    }

    /// is session closed?
    #[inline(always)]
    pub fn closed(&self) -> bool {
        self.0.session_handle().is_none()
    }
}

#[derive(Debug)]
pub struct State<T> {
    pub(crate) session_handle: Option<u32>,
    pub(crate) service: Framed<T, ClientCodec>,
}

impl<T> Context for State<T>
where
    T: AsyncRead + AsyncWrite,
{
    type Stream = T;

    #[inline(always)]
    fn from_stream(stream: Self::Stream) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(State {
            session_handle: None,
            service: Framed::new(stream, ClientCodec::default()),
        })
    }

    #[inline(always)]
    fn session_handle(&self) -> Option<u32> {
        self.session_handle
    }

    #[inline(always)]
    fn session_handle_mut(&mut self) -> &mut Option<u32> {
        &mut self.session_handle
    }

    #[inline(always)]
    fn framed(&self) -> &Framed<T, ClientCodec> {
        &self.service
    }

    #[inline(always)]
    fn framed_mut(&mut self) -> &mut Framed<T, ClientCodec> {
        &mut self.service
    }
}

impl<T> TcpService for State<T> where T: AsyncRead + AsyncWrite {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        frame::cip::{PortSegment, Segment},
        test::block_on,
    };
    use byteorder::{ByteOrder, LittleEndian};
    use bytes::{BufMut, BytesMut};

    //#[test]
    fn ab_read_tag() {
        block_on(async {
            let connection_path = EPath::from(vec![Segment::Port(PortSegment::default())]);
            let mut client = Client::connect("192.168.0.83").await?;
            let mr_request = MessageRouterRequest::new(
                0x4c,
                EPath::from(vec![Segment::Symbol("test_car1_x".to_owned())]),
                ElementCount(1),
            );
            let resp = client.send(mr_request, connection_path).await?;
            assert_eq!(resp.reply_service, 0xCC); // read tag service reply
            assert_eq!(LittleEndian::read_u16(&resp.data[0..2]), 0xC4); // DINT
            client.close().await?;
            Ok(())
        });
    }

    struct ElementCount(u16);

    impl Encodable for ElementCount {
        fn encode(self, dst: &mut BytesMut) -> Result<()> {
            dst.put_u16_le(self.0);
            Ok(())
        }
        fn bytes_count(&self) -> usize {
            2
        }
    }
}
