// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::*;
use bytes::Bytes;
use core::marker::PhantomData;

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
    fn decode_sized<F, R>(&mut self, size: usize, f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnOnce(Self) -> Result<R, Self::Error>,
    {
        self.ensure_size(size)?;
        let buf = self.buf.split_to(size);
        let decoder = Self::new(buf);
        f(decoder)
    }
}
