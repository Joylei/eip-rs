// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod impls;
mod little_endian;
pub mod visitor;

use crate::Error;
use bytes::Buf;
pub use little_endian::LittleEndianDecoder;
pub use visitor::Visitor;

pub trait Decoder<'de> {
    type Error: Error;
    type Buf: Buf;
    /// inner buffer
    fn buf(&self) -> &Self::Buf;

    /// inner buffer
    fn buf_mut(&mut self) -> &mut Self::Buf;

    /// check remaining buffer size
    #[inline(always)]
    fn ensure_size(&self, expected: usize) -> Result<(), Self::Error> {
        let actual_len = self.buf().remaining();
        if actual_len < expected {
            Err(Self::Error::invalid_length(actual_len, expected))
        } else {
            Ok(())
        }
    }

    /// decode any type `T` that derive [`Decode`]
    #[inline]
    fn decode_any<T>(&mut self) -> Result<T, Self::Error>
    where
        T: Decode<'de>,
        Self: Sized,
    {
        T::decode(self)
    }

    /// get bool unchecked
    #[inline(always)]
    fn decode_bool(&mut self) -> bool {
        self.buf_mut().get_u8() != 0
    }

    /// get i8 unchecked
    #[inline(always)]
    fn decode_i8(&mut self) -> i8 {
        self.buf_mut().get_i8()
    }

    /// get u8 unchecked
    #[inline(always)]
    fn decode_u8(&mut self) -> u8 {
        self.buf_mut().get_u8()
    }

    /// get i16 unchecked
    #[inline(always)]
    fn decode_i16(&mut self) -> i16 {
        self.buf_mut().get_i16_le()
    }

    /// get u16 unchecked
    #[inline(always)]
    fn decode_u16(&mut self) -> u16 {
        self.buf_mut().get_u16_le()
    }

    /// get i32 unchecked
    #[inline(always)]
    fn decode_i32(&mut self) -> i32 {
        self.buf_mut().get_i32_le()
    }

    /// get u32 unchecked
    #[inline(always)]
    fn decode_u32(&mut self) -> u32 {
        self.buf_mut().get_u32_le()
    }

    /// get i64 unchecked
    #[inline(always)]
    fn decode_i64(&mut self) -> i64 {
        self.buf_mut().get_i64_le()
    }

    /// get u64 unchecked
    #[inline(always)]
    fn decode_u64(&mut self) -> u64 {
        self.buf_mut().get_u64_le()
    }

    /// get f32 unchecked
    #[inline(always)]
    fn decode_f32(&mut self) -> f32 {
        self.buf_mut().get_f32_le()
    }

    /// get f64 unchecked
    #[inline(always)]
    fn decode_f64(&mut self) -> f64 {
        self.buf_mut().get_f64_le()
    }

    /// get i128 unchecked
    #[inline(always)]
    fn decode_i128(&mut self) -> i128 {
        self.buf_mut().get_i128_le()
    }

    /// get u128 unchecked
    #[inline(always)]
    fn decode_u128(&mut self) -> u128 {
        self.buf_mut().get_u128_le()
    }

    #[inline(always)]
    fn remaining(&mut self) -> usize {
        self.buf().remaining()
    }

    #[inline(always)]
    fn has_remaining(&mut self) -> bool {
        self.buf().has_remaining()
    }

    /// decode with a dedicated [`Visitor`]. A [`Visitor`] gives you some context information while decoding.
    #[inline]
    fn decode_with<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        Self: Sized,
    {
        visitor.visit(self)
    }

    /// take specified number of bytes, and decode it with the specified visitor
    fn decode_sized<V: Visitor<'de>>(
        &mut self,
        size: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        Self: Sized;
}

pub trait Decode<'de>: Sized {
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>;

    #[doc(hidden)]
    #[inline]
    fn decode_in_place<D>(decoder: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: Decoder<'de>,
    {
        *place = Self::decode(decoder)?;
        Ok(())
    }
}
