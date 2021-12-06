// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

pub mod decode;
pub mod encode;

pub use decode::*;
pub use encode::*;

use bytes::{Buf, Bytes};

/// take remaining bytes
pub struct BytesHolder(Bytes);

impl From<BytesHolder> for Bytes {
    fn from(src: BytesHolder) -> Self {
        src.0
    }
}

impl<'de> Decode<'de> for BytesHolder {
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let size = decoder.remaining();
        let data = decoder.buf_mut().copy_to_bytes(size);
        Ok(Self(data))
    }
}
