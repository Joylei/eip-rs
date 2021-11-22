// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

pub mod decode;
pub mod encode;

use crate::Result;
use bytes::{Bytes, BytesMut};

#[derive(Debug, Default, PartialEq)]
pub struct ClientCodec {}

pub trait Encodable {
    fn encode(self, dst: &mut BytesMut) -> Result<()>;

    /// encoded bytes count
    fn bytes_count(&self) -> usize;

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
