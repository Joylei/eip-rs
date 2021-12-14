// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::*;
use bytes::Bytes;
use core::marker::PhantomData;

#[derive(Debug)]
pub struct LittleEndianDecoder<E> {
    buf: Bytes,
    _marker: PhantomData<E>,
}

impl<E> LittleEndianDecoder<E> {
    pub fn new(buf: Bytes) -> Self {
        Self {
            buf,
            _marker: Default::default(),
        }
    }

    pub fn into_inner(self) -> Bytes {
        self.buf
    }
}

impl<'de, E: Error> Decoder<'de> for LittleEndianDecoder<E> {
    type Buf = Bytes;
    type Error = E;

    #[inline(always)]
    fn buf(&self) -> &Self::Buf {
        &self.buf
    }

    #[inline(always)]
    fn buf_mut(&mut self) -> &mut Self::Buf {
        &mut self.buf
    }

    #[inline]
    fn decode_sized<V: Visitor<'de>>(
        &mut self,
        size: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        Self: Sized,
    {
        self.ensure_size(size)?;
        let buf = self.buf.split_to(size);
        let decoder = Self::new(buf);
        visitor.visit(decoder)
    }
}
