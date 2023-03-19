use std::{
    io,
    net::{SocketAddr, UdpSocket as StdUdpSocket},
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

pub trait AsyncUdpSocket: Unpin + Send + 'static {
    fn from_std(socket: StdUdpSocket) -> io::Result<Self>
    where
        Self: Sized;
    fn poll_read(
        &mut self,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<(usize, SocketAddr)>>;

    fn poll_write(&mut self, cx: &mut Context, buf: &[u8], to: SocketAddr) -> Poll<io::Result<()>>;

    fn split(self) -> (AsyncUdpReadHalf<Self>, AsyncUdpWriteHalf<Self>)
    where
        Self: Sized,
    {
        let inner = Arc::new(Mutex::new(self));
        (
            AsyncUdpReadHalf {
                inner: inner.clone(),
            },
            AsyncUdpWriteHalf { inner },
        )
    }
}

// TODO: improve it

#[derive(Debug)]
pub struct AsyncUdpReadHalf<S> {
    inner: Arc<Mutex<S>>,
}

#[derive(Debug)]
pub struct AsyncUdpWriteHalf<S> {
    inner: Arc<Mutex<S>>,
}

impl<S: AsyncUdpSocket> AsyncUdpReadHalf<S> {
    #[inline]
    pub fn poll_read(
        &mut self,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<(usize, SocketAddr)>> {
        loop {
            if let Ok(mut guard) = self.inner.try_lock() {
                return guard.poll_read(cx, buf);
            }
        }
    }
}

impl<S: AsyncUdpSocket> AsyncUdpWriteHalf<S> {
    #[inline]
    pub fn poll_write(
        &mut self,
        cx: &mut Context,
        buf: &[u8],
        to: SocketAddr,
    ) -> Poll<io::Result<()>> {
        loop {
            if let Ok(mut guard) = self.inner.try_lock() {
                return guard.poll_write(cx, buf, to);
            }
        }
    }
}
