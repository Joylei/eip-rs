// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::*;
use core::marker::PhantomData;
use core::mem;

impl<'t, 'de, T: Decoder<'de>> Decoder<'de> for &'t mut T {
    type Error = T::Error;
    type Buf = T::Buf;

    #[inline]
    fn buf(&self) -> &Self::Buf {
        (**self).buf()
    }

    #[inline]
    fn buf_mut(&mut self) -> &mut Self::Buf {
        (**self).buf_mut()
    }

    #[inline]
    fn decode_sized<F, R>(&mut self, size: usize, f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnOnce(Self) -> Result<R, Self::Error>,
    {
        (**self).decode_sized(size, |mut x| {
            // problem here?
            let x: &'t mut T = unsafe { mem::transmute(&mut x) };
            f(x)
        })
    }
}

macro_rules! impl_primitive {
    ($ty:ty, $m: tt) => {
        impl<'de> Decode<'de> for $ty {
            #[inline(always)]
            fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                decoder.ensure_size(mem::size_of::<Self>())?;
                Ok(decoder.$m())
            }
        }
    };
}

impl_primitive!(bool, decode_bool);
impl_primitive!(i8, decode_i8);
impl_primitive!(u8, decode_u8);
impl_primitive!(i16, decode_i16);
impl_primitive!(u16, decode_u16);
impl_primitive!(i32, decode_i32);
impl_primitive!(u32, decode_u32);
impl_primitive!(i64, decode_i64);
impl_primitive!(u64, decode_u64);
impl_primitive!(f32, decode_f32);
impl_primitive!(f64, decode_f64);
impl_primitive!(i128, decode_i128);
impl_primitive!(u128, decode_u128);

impl<'de> Decode<'de> for () {
    #[inline]
    fn decode<D>(_decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(())
    }
}

impl<'de, T> Decode<'de> for PhantomData<T>
where
    T: 'de,
{
    #[inline]
    fn decode<D>(_decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(Default::default())
    }
}

impl<'de, T> Decode<'de> for Option<T>
where
    T: Decode<'de>,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let v = T::decode(decoder)?;
        Ok(Some(v))
    }
}

// impl<'de> Decode<'de> for Bytes {
//     #[inline]
//     fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
//     where
//         D: Decoder<'de>,
//     {
//         let size = decoder.remaining();
//         Ok(decoder.buf_mut().copy_to_bytes(size))
//     }
// }

impl<'de, T0, T1> Decode<'de> for (T0, T1)
where
    T0: Decode<'de>,
    T1: Decode<'de>,
{
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let v0 = decoder.decode_any()?;
        let v1 = decoder.decode_any()?;
        Ok((v0, v1))
    }
}
