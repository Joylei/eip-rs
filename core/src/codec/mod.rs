// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

/*！
The library provides `Encode` to encode values, `Decode` to decode values, and `TagValue` to manipulate tag data values. The library already implements `Encode` and `Decode` for some rust types: `bool`,`i8`,`u8`,`i16`,`u16`,`i32`,`u32`,`i64`,`u64`,`f32`,`f64`,`i128`,`u128`,`()`,`Option`,`Tuple`,`Vec`,`[T;N]`,`SmallVec`. For structure type, you need to implement `Encode` and `Decode` by yourself.
*/

pub mod decode;
pub mod encode;

pub use decode::*;
pub use encode::*;

use bytes::{Buf, Bytes};

/// take remaining bytes
#[derive(Debug, Clone)]
pub struct BytesHolder(Bytes);

impl From<BytesHolder> for Bytes {
    #[inline]
    fn from(src: BytesHolder) -> Self {
        src.0
    }
}

impl<'de> Decode<'de> for BytesHolder {
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let size = decoder.remaining();
        let data = decoder.buf_mut().copy_to_bytes(size);
        Ok(Self(data))
    }
}
