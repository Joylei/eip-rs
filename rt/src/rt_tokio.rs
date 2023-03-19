use crate::{AsyncUdpSocket, Runtime};
use futures_util::{future::BoxFuture, ready, AsyncRead, AsyncWrite};
use pin_project_lite::pin_project;
use std::{
    io,
    net::{SocketAddr, SocketAddrV4},
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    net::{lookup_host, TcpSocket, TcpStream},
    time::Sleep,
};

pin_project! {
    pub struct TokioTcpStream {
        #[pin]
        inner: TcpStream,
    }
}

impl TokioTcpStream {
    #[inline]
    pub async fn connect(addr: SocketAddrV4) -> io::Result<Self> {
        let socket = TcpSocket::new_v4()?;
        let stream = socket.connect(addr.into()).await?;
        Ok(Self { inner: stream })
    }
}

impl AsyncRead for TokioTcpStream {
    #[inline]
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut buf = tokio::io::ReadBuf::new(buf);
        ready!(tokio::io::AsyncRead::poll_read(
            self.project().inner,
            cx,
            &mut buf
        ))?;
        Poll::Ready(Ok(buf.filled().len()))
    }
}

impl AsyncWrite for TokioTcpStream {
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        tokio::io::AsyncWrite::poll_write(self.project().inner, cx, buf)
    }
    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        tokio::io::AsyncWrite::poll_flush(self.project().inner, cx)
    }
    #[inline]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        tokio::io::AsyncWrite::poll_shutdown(self.project().inner, cx)
    }
}

pub struct TokioRuntime;

impl Runtime for TokioRuntime {
    type Transport = TokioTcpStream;
    type UdpSocket = tokio::net::UdpSocket;
    type Sleep = Sleep;
    fn lookup_host(host: String) -> BoxFuture<'static, io::Result<SocketAddrV4>> {
        Box::pin(async move {
            let addr = lookup_host(host)
                .await?
                .filter_map(|item| match item {
                    SocketAddr::V4(addr) => Some(addr),
                    _ => None,
                })
                .next();
            if let Some(addr) = addr {
                Ok(addr)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "dns lookup failure",
                ))
            }
        })
    }

    fn sleep(duration: std::time::Duration) -> Sleep {
        tokio::time::sleep(duration)
    }
}

impl AsyncUdpSocket for tokio::net::UdpSocket {
    fn from_std(socket: std::net::UdpSocket) -> io::Result<Self>
    where
        Self: Sized,
    {
        tokio::net::UdpSocket::from_std(socket)
    }
    #[inline]
    fn poll_read(
        &mut self,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<(usize, SocketAddr)>> {
        let mut buf = tokio::io::ReadBuf::new(buf);
        let addr = ready!(self.poll_recv_from(cx, &mut buf))?;
        Poll::Ready(Ok((buf.filled().len(), addr)))
    }
    #[inline]
    fn poll_write(&mut self, cx: &mut Context, buf: &[u8], to: SocketAddr) -> Poll<io::Result<()>> {
        ready!(self.poll_send_to(cx, buf, to))?;
        Poll::Ready(Ok(()))
    }
}
