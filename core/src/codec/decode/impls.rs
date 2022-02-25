// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

#![allow(non_snake_case)]

use smallvec::SmallVec;

use super::*;
use core::marker::PhantomData;
use std::mem;

impl<'de, T: Decoder<'de>> Decoder<'de> for &mut T {
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
    fn decode_sized<V: Visitor<'de>>(
        &mut self,
        size: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        Self: Sized,
    {
        (**self).decode_sized(size, visitor)
    }
}

macro_rules! impl_primitive {
    ($ty:ty, $m:tt, $s:tt) => {
        impl<'de> Decode<'de> for $ty {
            #[inline]
            fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                decoder.ensure_size($s)?;
                Ok(decoder.$m())
            }
        }
    };
}

impl_primitive!(bool, decode_bool, 1);
impl_primitive!(i8, decode_i8, 1);
impl_primitive!(u8, decode_u8, 1);
impl_primitive!(i16, decode_i16, 2);
impl_primitive!(u16, decode_u16, 2);
impl_primitive!(i32, decode_i32, 4);
impl_primitive!(u32, decode_u32, 4);
impl_primitive!(i64, decode_i64, 8);
impl_primitive!(u64, decode_u64, 8);
impl_primitive!(f32, decode_f32, 4);
impl_primitive!(f64, decode_f64, 8);
impl_primitive!(i128, decode_i128, 16);
impl_primitive!(u128, decode_u128, 16);

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

impl<'de, T, const N: usize> Decode<'de> for [T; N]
where
    T: Decode<'de> + Default,
{
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut buffer = mem::MaybeUninit::<[T; N]>::uninit();
        {
            let buffer = unsafe { &mut *buffer.as_mut_ptr() };
            for item in buffer.iter_mut() {
                if decoder.has_remaining() {
                    T::decode_in_place(&mut decoder, item)?;
                } else {
                    *item = Default::default();
                }
            }
        }
        Ok(unsafe { buffer.assume_init() })
    }
}

impl<'de, T> Decode<'de> for Vec<T>
where
    T: Decode<'de>,
{
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut res = Vec::new();
        while decoder.has_remaining() {
            let v = T::decode(&mut decoder)?;
            res.push(v);
        }
        Ok(res)
    }
}

impl<'de, A> Decode<'de> for SmallVec<A>
where
    A: smallvec::Array,
    A::Item: Decode<'de>,
{
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut res = Self::new();
        while decoder.has_remaining() {
            let v = A::Item::decode(&mut decoder)?;
            res.push(v);
        }
        Ok(res)
    }
}

macro_rules! impl_tuple {
    ($($n:tt $name:ident)+) => {
        impl<'de, $($name,)+> Decode<'de> for ($($name,)+)
        where
            $($name: Decode<'de>,)+
        {
            #[inline]
            fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                $(
                    let $name = decoder.decode_any()?;
                )+
                Ok(($($name,)+))
            }
        }
    }
}

impl_tuple!(0 T0);
impl_tuple!(0 T0 1 T1);
impl_tuple!(0 T0 1 T1 2 T2);
impl_tuple!(0 T0 1 T1 2 T2 3 T3);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15);
