mod cip;
mod command;
mod common_packet;
mod connected_send;
mod encapsulation;
mod epath;
mod message_request;
mod unconnected_send;

use super::{ClientCodec, Encodable};
use crate::error::Error;
use bytes::{BufMut, Bytes, BytesMut};
use tokio_util::codec::Encoder;

impl Encodable for () {
    #[inline(always)]
    fn encode(self, _: &mut bytes::BytesMut) -> crate::Result<()> {
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        0
    }
}

impl<D1, D2> Encodable for (D1, D2)
where
    D1: Encodable,
    D2: Encodable,
{
    #[inline(always)]
    fn encode(self, dst: &mut bytes::BytesMut) -> crate::Result<()> {
        self.0.encode(dst)?;
        self.1.encode(dst)?;
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.0.bytes_count() + self.1.bytes_count()
    }
}

impl<D1, D2, D3> Encodable for (D1, D2, D3)
where
    D1: Encodable,
    D2: Encodable,
    D3: Encodable,
{
    #[inline(always)]
    fn encode(self, dst: &mut bytes::BytesMut) -> crate::Result<()> {
        self.0.encode(dst)?;
        self.1.encode(dst)?;
        self.2.encode(dst)?;
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.0.bytes_count() + self.1.bytes_count() + self.2.bytes_count()
    }
}

impl Encodable for &[u8] {
    #[inline(always)]
    fn encode(self, dst: &mut bytes::BytesMut) -> crate::Result<()> {
        dst.put_slice(self);
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.len()
    }
}

#[derive(Debug)]
pub struct LazyEncode<F> {
    pub f: F,
    pub bytes_count: usize,
}

impl<F> Encodable for LazyEncode<F>
where
    F: FnOnce(&mut bytes::BytesMut) -> crate::Result<()> + Send,
{
    #[inline(always)]
    fn encode(self, dst: &mut bytes::BytesMut) -> crate::Result<()> {
        (self.f)(dst)
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.bytes_count
    }
}

impl<E: Encodable> Encoder<E> for ClientCodec {
    type Error = Error;
    #[inline(always)]
    fn encode(&mut self, item: E, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        item.encode(dst)
    }
}

impl Encodable for Bytes {
    #[inline(always)]
    fn encode(self, dst: &mut bytes::BytesMut) -> Result<(), Error> {
        dst.put_slice(&self);
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.len()
    }
}

impl<D: Encodable> Encodable for Option<D> {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> crate::Result<()> {
        if let Some(item) = self {
            item.encode(dst)
        } else {
            Ok(())
        }
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.as_ref().map(|v| v.bytes_count()).unwrap_or_default()
    }
}
