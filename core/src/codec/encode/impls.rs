// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::*;
use bytes::{BufMut, Bytes, BytesMut};
use core::{marker::PhantomData, mem};
use smallvec::SmallVec;

impl<A: Encoder> Encoder for &mut A {
    type Error = A::Error;

    fn encode_bool(&mut self, item: bool, buf: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode_bool(item, buf)
    }

    fn encode_i8(&mut self, item: i8, buf: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode_i8(item, buf)
    }

    fn encode_u8(&mut self, item: u8, buf: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode_u8(item, buf)
    }

    fn encode_i16(&mut self, item: i16, buf: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode_i16(item, buf)
    }

    fn encode_u16(&mut self, item: u16, buf: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode_u16(item, buf)
    }

    fn encode_i32(&mut self, item: i32, buf: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode_i32(item, buf)
    }

    fn encode_u32(&mut self, item: u32, buf: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode_u32(item, buf)
    }

    fn encode_i64(&mut self, item: i64, buf: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode_i64(item, buf)
    }

    fn encode_u64(&mut self, item: u64, buf: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode_u64(item, buf)
    }

    fn encode_f32(&mut self, item: f32, buf: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode_f32(item, buf)
    }

    fn encode_f64(&mut self, item: f64, buf: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode_f64(item, buf)
    }

    fn encode_i128(&mut self, item: i128, buf: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode_i128(item, buf)
    }

    fn encode_u128(&mut self, item: u128, buf: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode_u128(item, buf)
    }
}

macro_rules! impl_primitive {
    ($ty: ident) => {
        paste::paste! {
            impl Encode for $ty {
                #[inline]
                fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
                where
                    Self: Sized,
                {
                    encoder.[<encode_ $ty>](self, buf)
                }

                #[inline]
                fn encode_by_ref<A: Encoder>(
                    &self,
                    buf: &mut BytesMut,
                    encoder: &mut A,
                ) -> Result<(), A::Error> {
                    encoder.[<encode_ $ty>](*self, buf)
                }

                #[inline(always)]
                fn bytes_count(&self)->usize {
                    mem::size_of::<$ty>()
                }
            }
        }
    };
}

impl_primitive!(bool);
impl_primitive!(i8);
impl_primitive!(u8);
impl_primitive!(i16);
impl_primitive!(u16);
impl_primitive!(i32);
impl_primitive!(u32);
impl_primitive!(i64);
impl_primitive!(u64);
impl_primitive!(i128);
impl_primitive!(u128);

impl<T: Encode> Encode for &T {
    #[inline]
    fn encode_by_ref<A: Encoder>(&self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        (**self).encode_by_ref(buf, encoder)
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        (**self).bytes_count()
    }
}

impl Encode for () {
    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        _buf: &mut BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        0
    }
}

impl<T: Encode> Encode for PhantomData<T> {
    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        _buf: &mut BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        0
    }
}

impl<T: Encode> Encode for Option<T> {
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        if let Some(item) = self {
            item.encode(buf, encoder)?;
        }
        Ok(())
    }

    #[inline]
    fn encode_by_ref<A: Encoder>(&self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        if let Some(item) = self {
            item.encode_by_ref(buf, encoder)?;
        }
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.as_ref().map(|v| v.bytes_count()).unwrap_or_default()
    }
}

impl<T: Encode, const N: usize> Encode for [T; N] {
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        for item in self {
            item.encode(buf, encoder)?;
        }
        Ok(())
    }

    #[inline]
    fn encode_by_ref<A: Encoder>(&self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        for item in self {
            item.encode_by_ref(buf, encoder)?;
        }
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        self.iter().map(|v| v.bytes_count()).sum()
    }
}

impl<T> Encode for SmallVec<T>
where
    T: smallvec::Array,
    T::Item: Encode,
{
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        for item in self {
            item.encode(buf, encoder)?;
        }
        Ok(())
    }

    #[inline]
    fn encode_by_ref<A: Encoder>(&self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        for item in self {
            item.encode_by_ref(buf, encoder)?;
        }
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        self.iter().map(|v| v.bytes_count()).sum()
    }
}

impl<T: Encode> Encode for Vec<T> {
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        for item in self {
            item.encode(buf, encoder)?;
        }
        Ok(())
    }

    #[inline]
    fn encode_by_ref<A: Encoder>(&self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        for item in self {
            item.encode_by_ref(buf, encoder)?;
        }
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        self.iter().map(|v| v.bytes_count()).sum()
    }
}

impl Encode for Bytes {
    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error> {
        buf.put_slice(self);
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.len()
    }
}

impl Encode for &[u8] {
    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error> {
        buf.put_slice(self);
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.len()
    }
}

pub struct SliceContainer<'a, T> {
    inner: &'a [T],
    bytes_count: Option<usize>,
}

impl<'a, T> SliceContainer<'a, T> {
    #[inline]
    pub fn new(inner: &'a [T]) -> Self {
        Self {
            inner,
            bytes_count: None,
        }
    }

    /// fast path to compute number of bytes
    #[inline]
    pub fn with_bytes_count(mut self, size: usize) -> Self {
        self.bytes_count = Some(size);
        self
    }
}

impl<'a, T> Encode for SliceContainer<'a, T>
where
    T: Encode,
{
    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        for item in self.inner.iter() {
            item.encode_by_ref(buf, encoder)?;
        }
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        if let Some(v) = self.bytes_count {
            v
        } else {
            self.inner.iter().map(|v| v.bytes_count()).sum()
        }
    }
}

impl<T0, T1> Encode for (T0, T1)
where
    T0: Encode,
    T1: Encode,
{
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        self.0.encode(buf, encoder)?;
        self.1.encode(buf, encoder)?;
        Ok(())
    }

    #[inline]
    fn encode_by_ref<A: Encoder>(&self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        self.0.encode_by_ref(buf, encoder)?;
        self.1.encode_by_ref(buf, encoder)?;
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        self.0.bytes_count() + self.1.bytes_count()
    }
}
