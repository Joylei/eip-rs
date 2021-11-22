// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod cip;
mod command;
mod common_packet;
mod connection;
mod encapsulation;
mod epath;
mod message_request;

use super::{ClientCodec, Encodable};
use crate::{error::Error, Result};
use bytes::{BufMut, Bytes, BytesMut};
use std::fmt;
use tokio_util::codec::Encoder;

impl Encodable for () {
    #[inline(always)]
    fn encode(self, _: &mut BytesMut) -> Result<()> {
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
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
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
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
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
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        dst.put_slice(self);
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.len()
    }
}

impl<T: Encodable> Encodable for Vec<T> {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        for item in self {
            item.encode(dst)?;
        }
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.iter().map(|v| v.bytes_count()).sum()
    }
}

impl<E: Encodable> Encoder<E> for ClientCodec {
    type Error = Error;
    #[inline(always)]
    fn encode(&mut self, item: E, dst: &mut BytesMut) -> Result<()> {
        item.encode(dst)
    }
}

impl Encodable for Bytes {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
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
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
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

// conflict with TryFrom<Bytes> for Bytes
// impl<T: Encodable> TryFrom<T> for Bytes {
//     type Error = Error;
//     fn try_from(src: T) -> Result<Self> {
//         let mut buf = BytesMut::new();
//         src.encode(&mut buf)?;
//         Ok(buf.freeze())
//     }
// }

impl<T: Encodable + Sized> Encodable for Box<T> {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        (*self).encode(dst)
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        (&**self).bytes_count()
    }
}

pub trait SizedEncodable: Encodable + Sized {}

impl<T: Encodable + Sized> SizedEncodable for T {}

//#[derive(Debug)]
pub struct LazyEncode<F> {
    pub f: F,
    pub bytes_count: usize,
}

impl<F> Encodable for LazyEncode<F>
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

impl<F> LazyEncode<F>
where
    F: FnOnce(&mut BytesMut) -> Result<()> + 'static,
{
    #[inline(always)]
    pub fn into_dynamic(self) -> DynamicEncode {
        DynamicEncode {
            f: Box::new(self.f),
            bytes_count: self.bytes_count,
        }
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

pub type DynamicEncode = LazyEncode<Box<dyn FnOnce(&mut BytesMut) -> Result<()>>>;

impl Default for DynamicEncode {
    #[inline]
    fn default() -> Self {
        Self {
            f: Box::new(|_| Ok(())),
            bytes_count: 0,
        }
    }
}

impl<T: AsRef<[u8]> + 'static> From<T> for DynamicEncode {
    #[inline]
    fn from(src: T) -> Self {
        let bytes_count = src.as_ref().len();
        Self {
            f: Box::new(move |buf| {
                buf.put_slice(src.as_ref());
                Ok(())
            }),
            bytes_count,
        }
    }
}
