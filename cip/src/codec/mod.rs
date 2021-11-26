// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod decode;
mod encode;

use crate::Result;
use bytes::{Bytes, BytesMut};
pub use encode::*;

/// encoding
pub trait Encodable {
    /// encode current object to [`BytesMut`]
    fn encode(self, dst: &mut BytesMut) -> Result<()>;

    /// encoded bytes count
    fn bytes_count(&self) -> usize;

    #[doc(hidden)]
    #[inline]
    fn try_into_bytes(self) -> Result<Bytes>
    where
        Self: Sized,
    {
        let mut buf = BytesMut::new();
        self.encode(&mut buf)?;
        Ok(buf.freeze())
    }
}
