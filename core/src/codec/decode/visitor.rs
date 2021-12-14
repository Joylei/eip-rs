// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::*;
use crate::Either;
use core::marker::PhantomData;

pub trait Visitor<'de> {
    type Value;
    fn visit<D: Decoder<'de>>(self, decoder: D) -> Result<Self::Value, D::Error>;

    #[inline]
    fn map<F, R>(self, f: F) -> MapVisitor<'de, Self, F, R>
    where
        Self: Sized,
        F: FnOnce(Self::Value) -> R,
    {
        MapVisitor {
            v: self,
            f,
            _marker: Default::default(),
        }
    }

    #[inline]
    fn and<A>(self, visitor: A) -> AndVisitor<'de, Self, A>
    where
        Self: Sized,
        A: Visitor<'de>,
    {
        AndVisitor {
            a: self,
            b: visitor,
            _marker: Default::default(),
        }
    }

    #[inline]
    fn or<A>(self, visitor: A) -> EitherVisitor<'de, Self, A>
    where
        Self: Sized,
        A: Visitor<'de>,
    {
        EitherVisitor {
            a: self,
            b: visitor,
            _marker: Default::default(),
        }
    }
}

/// decode any type
#[inline]
pub fn any<'a, R>() -> AnyVisitor<'a, R> {
    AnyVisitor(Default::default())
}

/// directly returns specified value
#[inline]
pub fn from_value<R>(v: R) -> FromValueVisitor<R> {
    FromValueVisitor(v)
}

#[derive(Debug)]
pub struct FromValueVisitor<R>(R);

impl<'de, R> Visitor<'de> for FromValueVisitor<R> {
    type Value = R;

    #[inline(always)]
    fn visit<D: Decoder<'de>>(self, _decoder: D) -> Result<Self::Value, D::Error> {
        Ok(self.0)
    }
}

#[derive(Debug)]
pub struct AnyVisitor<'de, R>(PhantomData<&'de R>);

impl<'de, R: Decode<'de>> Visitor<'de> for AnyVisitor<'de, R> {
    type Value = R;

    #[inline]
    fn visit<D: Decoder<'de>>(self, mut decoder: D) -> Result<Self::Value, D::Error> {
        decoder.decode_any()
    }
}

#[derive(Debug)]
pub struct MapVisitor<'de, V, F, R>
where
    V: Visitor<'de>,
    F: FnOnce(V::Value) -> R,
{
    v: V,
    f: F,
    _marker: PhantomData<&'de R>,
}

impl<'de, V, F, R> Visitor<'de> for MapVisitor<'de, V, F, R>
where
    V: Visitor<'de>,
    F: FnOnce(V::Value) -> R,
{
    type Value = R;
    #[inline]
    fn visit<D: Decoder<'de>>(self, decoder: D) -> Result<Self::Value, D::Error> {
        let prev = self.v.visit(decoder)?;
        let v = (self.f)(prev);
        Ok(v)
    }
}

#[derive(Debug)]
pub struct AndVisitor<'de, A, B>
where
    A: Visitor<'de>,
    B: Visitor<'de>,
{
    a: A,
    b: B,
    _marker: PhantomData<&'de A>,
}

impl<'de, A, B> Visitor<'de> for AndVisitor<'de, A, B>
where
    A: Visitor<'de>,
    B: Visitor<'de>,
{
    type Value = (A::Value, B::Value);
    #[inline]
    fn visit<D: Decoder<'de>>(self, mut decoder: D) -> Result<Self::Value, D::Error> {
        let a = self.a.visit(&mut decoder)?;
        let b = self.b.visit(decoder)?;
        Ok((a, b))
    }
}

#[derive(Debug)]
pub struct EitherVisitor<'de, A, B>
where
    A: Visitor<'de>,
    B: Visitor<'de>,
{
    a: A,
    b: B,
    _marker: PhantomData<&'de A>,
}

impl<'de, A, B> Visitor<'de> for EitherVisitor<'de, A, B>
where
    A: Visitor<'de>,
    B: Visitor<'de>,
{
    type Value = Either<A::Value, B::Value>;
    #[inline]
    fn visit<D: Decoder<'de>>(self, mut decoder: D) -> Result<Self::Value, D::Error> {
        if let Ok(v) = self.a.visit(&mut decoder) {
            Ok(Either::Left(v))
        } else {
            let v = self.b.visit(decoder)?;
            Ok(Either::Right(v))
        }
    }
}
