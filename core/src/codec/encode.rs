// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod impls;

use crate::Error;
use bytes::BytesMut;
#[doc(inline)]
pub use impls::SliceContainer;

pub trait Encoder {
    type Error: Error;

    fn encode_bool(&mut self, item: bool, buf: &mut BytesMut) -> Result<(), Self::Error>;
    fn encode_i8(&mut self, item: i8, buf: &mut BytesMut) -> Result<(), Self::Error>;
    fn encode_u8(&mut self, item: u8, buf: &mut BytesMut) -> Result<(), Self::Error>;
    fn encode_i16(&mut self, item: i16, buf: &mut BytesMut) -> Result<(), Self::Error>;
    fn encode_u16(&mut self, item: u16, buf: &mut BytesMut) -> Result<(), Self::Error>;
    fn encode_i32(&mut self, item: i32, buf: &mut BytesMut) -> Result<(), Self::Error>;
    fn encode_u32(&mut self, item: u32, buf: &mut BytesMut) -> Result<(), Self::Error>;
    fn encode_i64(&mut self, item: i64, buf: &mut BytesMut) -> Result<(), Self::Error>;
    fn encode_u64(&mut self, item: u64, buf: &mut BytesMut) -> Result<(), Self::Error>;
    fn encode_f32(&mut self, item: f32, buf: &mut BytesMut) -> Result<(), Self::Error>;
    fn encode_f64(&mut self, item: f64, buf: &mut BytesMut) -> Result<(), Self::Error>;
    fn encode_i128(&mut self, item: i128, buf: &mut BytesMut) -> Result<(), Self::Error>;
    fn encode_u128(&mut self, item: u128, buf: &mut BytesMut) -> Result<(), Self::Error>;

    #[inline]
    fn encode<T>(&mut self, item: T, buf: &mut BytesMut) -> Result<(), Self::Error>
    where
        T: Encode + Sized,
        Self: Sized,
    {
        item.encode(buf, self)
    }

    #[inline]
    fn encode_by_ref<T>(&mut self, item: &T, buf: &mut BytesMut) -> Result<(), Self::Error>
    where
        T: Encode + ?Sized,
        Self: Sized,
    {
        item.encode_by_ref(buf, self)
    }
}

pub trait Encode {
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        self.encode_by_ref(buf, encoder)
    }

    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error>;

    fn bytes_count(&self) -> usize;
}
