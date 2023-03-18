// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::EncapsulationPacket;
use crate::consts::EIP_COMMAND_NOP;
use asynchronous_codec::{self as codec, Decoder, Encoder};
use bytes::Bytes;
use futures_util::{AsyncRead, AsyncWrite};
use futures_util::{Sink, Stream};
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

/// special Framed for EIP,
/// will ignore NOP from received packets
pub(crate) struct Framed<T, U> {
    inner: codec::Framed<T, U>,
}

impl<T, U> Framed<T, U>
where
    T: AsyncRead + AsyncWrite,
    U: Encoder + Decoder,
{
    pub fn new(inner: T, codec: U) -> Self {
        Self {
            inner: codec::Framed::new(inner, codec),
        }
    }
    #[allow(unused)]
    pub fn codec(&self) -> &U {
        self.inner.codec()
    }
    #[allow(unused)]
    pub fn codec_mut(&mut self) -> &mut U {
        self.inner.codec_mut()
    }
}

impl<T, U> Stream for Framed<T, U>
where
    T: AsyncRead + Unpin,
    U: Decoder<Item = EncapsulationPacket<Bytes>>,
{
    type Item = Result<U::Item, U::Error>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let inner = Pin::new(&mut self.get_mut().inner);
        match inner.poll_next(cx) {
            Poll::Ready(Some(Ok(item))) => {
                if item.hdr.command == EIP_COMMAND_NOP {
                    Poll::Pending
                } else {
                    Poll::Ready(Some(Ok(item)))
                }
            }
            v => v,
        }
    }
}

impl<T, I, U> Sink<I> for Framed<T, U>
where
    T: AsyncWrite + Unpin,
    U: Encoder<Item = I>,
    U::Error: From<io::Error>,
{
    type Error = U::Error;

    #[inline]
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let inner = Pin::new(&mut self.get_mut().inner);
        inner.poll_ready(cx)
    }

    #[inline]
    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        let inner = Pin::new(&mut self.get_mut().inner);
        inner.start_send(item)
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let inner = Pin::new(&mut self.get_mut().inner);
        inner.poll_flush(cx)
    }

    #[inline]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let inner = Pin::new(&mut self.get_mut().inner);
        inner.poll_close(cx)
    }
}
