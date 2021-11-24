// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod decode;
mod encode;

use crate::{Result, StdResult};
use bytes::{BufMut, Bytes, BytesMut};
use std::{fmt, io};

#[derive(Debug, Default, PartialEq)]
pub struct ClientCodec {}

pub trait Encoding {
    fn bytes_count(&self) -> usize;
    fn encode(self, buf: &mut BytesMut) -> Result<()>;

    #[inline(always)]
    fn try_into_bytes(self) -> Result<Bytes>
    where
        Self: Sized,
    {
        let mut buf = BytesMut::new();
        self.encode(&mut buf)?;
        Ok(buf.freeze())
    }
}

impl Encoding for () {
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        0
    }
    #[inline(always)]
    fn encode(self, _buf: &mut BytesMut) -> Result<()> {
        Ok(())
    }
}

impl Encoding for &[u8] {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        dst.put_slice(self);
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.len()
    }
}

pub(crate) struct LazyEncode<F> {
    pub f: F,
    pub bytes_count: usize,
}

impl<F> Encoding for LazyEncode<F>
where
    F: FnOnce(&mut BytesMut) -> Result<()>,
{
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        (self.f)(dst)
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.bytes_count
    }
}

impl<F> fmt::Debug for LazyEncode<F> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LazyEncode")
            .field("f", &"closure..")
            .field("bytes_count", &self.bytes_count)
            .finish()
    }
}

pub struct Frame<F, E>
where
    F: FnOnce(&mut BytesMut) -> StdResult<(), E>,
{
    pub(crate) bytes_count: usize,
    pub(crate) f: F,
}

impl<F, E> Frame<F, E>
where
    F: FnOnce(&mut BytesMut) -> StdResult<(), E>,
{
    #[inline]
    pub fn new(bytes_count: usize, f: F) -> Self {
        Self { bytes_count, f }
    }
}

impl<F, E> Encoding for Frame<F, E>
where
    F: FnOnce(&mut BytesMut) -> StdResult<(), E>,
{
    #[inline]
    fn encode(self, buf: &mut BytesMut) -> Result<()> {
        (self.f)(buf).map_err(|_| io::Error::new(io::ErrorKind::Other, "encoding failure").into())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.bytes_count
    }
}
