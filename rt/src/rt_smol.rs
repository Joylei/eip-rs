use crate::AsyncUdpSocket;
use crate::Runtime;
use async_io::{Async, Timer};
use async_net::{resolve, TcpStream};
use futures_util::{future::BoxFuture, ready};
use std::{
    io,
    net::{SocketAddr, SocketAddrV4, UdpSocket as StdUdpSocket},
    task::{Context, Poll},
};

pub struct SmolRuntime;

impl Runtime for SmolRuntime {
    type Transport = TcpStream;
    type UdpSocket = SmolUdpSocket;
    type Sleep = Timer;
    fn lookup_host(host: String) -> BoxFuture<'static, io::Result<SocketAddrV4>> {
        Box::pin(async move {
            let addr = resolve(host)
                .await?
                .into_iter()
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

    fn sleep(duration: std::time::Duration) -> Self::Sleep {
        Timer::after(duration)
    }
}

pub struct SmolUdpSocket {
    inner: Async<StdUdpSocket>,
}

impl AsyncUdpSocket for SmolUdpSocket {
    fn from_std(socket: StdUdpSocket) -> io::Result<Self>
    where
        Self: Sized,
    {
        Async::new(socket).map(|inner| Self { inner })
    }
    #[inline]
    fn poll_read(
        &mut self,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<(usize, SocketAddr)>> {
        ready!(self.inner.poll_readable(cx))?;
        Poll::Ready(self.inner.as_mut().recv_from(buf))
    }
    #[inline]
    fn poll_write(
        &mut self,
        cx: &mut Context,
        buf: &[u8],
        addr: SocketAddr,
    ) -> Poll<io::Result<()>> {
        ready!(self.inner.poll_writable(cx))?;
        Poll::Ready(self.inner.as_mut().send_to(buf, addr).map(|_| ()))
    }
}
