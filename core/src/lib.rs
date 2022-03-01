// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

/*!
# rseip-core
core module for `rseip`, please look at [rseip project](https://github.com/Joylei/eip-rs) for more information.

## License

MIT
*/

//#![warn(missing_docs)]
#![allow(clippy::match_like_matches_macro)]

#[cfg_attr(feature = "no_std", macro_use)]
extern crate alloc;

pub extern crate smallvec;

#[cfg(feature = "cip")]
pub mod cip;
pub mod codec;
mod either;
mod error;
pub mod hex;
pub mod iter;
mod string;

pub use either::Either;
pub use error::{Error, StdError};
pub use string::*;

/// only for testing, not public visible
#[doc(hidden)]
#[cfg(feature = "cip")]
pub mod tests {
    use crate::{
        codec::{Encode, Encoder},
        Error,
    };
    use bytes::{BufMut, Bytes, BytesMut};
    use core::fmt::{self, Debug, Display};
    use std::io;

    pub trait EncodeExt: Encode {
        fn try_into_bytes(self) -> Result<Bytes, CodecError>
        where
            Self: Sized;
    }

    impl<T: Encode> EncodeExt for T {
        fn try_into_bytes(self) -> Result<Bytes, CodecError>
        where
            Self: Sized,
        {
            let mut buf = BytesMut::new();
            self.encode(&mut buf, &mut TestEncoder::default())?;
            Ok(buf.freeze())
        }
    }

    #[derive(Debug)]
    pub enum CodecError {
        Io(io::Error),
        Msg(String),
    }

    impl Error for CodecError {
        fn with_kind(self, _kind: &'static str) -> Self {
            self
        }
        fn custom<T: core::fmt::Display>(msg: T) -> Self {
            Self::Msg(msg.to_string())
        }
    }

    impl From<io::Error> for CodecError {
        fn from(e: io::Error) -> Self {
            Self::Io(e)
        }
    }

    impl std::error::Error for CodecError {}

    impl Display for CodecError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Io(e) => write!(f, "{}", e),
                Self::Msg(e) => write!(f, "{}", e),
            }
        }
    }

    #[derive(Debug, Default)]
    pub struct TestEncoder {}

    impl Encoder for TestEncoder {
        type Error = CodecError;

        fn encode_bool(
            &mut self,
            item: bool,
            buf: &mut bytes::BytesMut,
        ) -> Result<(), Self::Error> {
            buf.put_u8(if item { 255 } else { 0 });
            Ok(())
        }

        fn encode_i8(&mut self, item: i8, buf: &mut bytes::BytesMut) -> Result<(), Self::Error> {
            buf.put_i8(item);
            Ok(())
        }

        fn encode_u8(&mut self, item: u8, buf: &mut bytes::BytesMut) -> Result<(), Self::Error> {
            buf.put_u8(item);
            Ok(())
        }

        fn encode_i16(&mut self, item: i16, buf: &mut bytes::BytesMut) -> Result<(), Self::Error> {
            buf.put_i16_le(item);
            Ok(())
        }

        fn encode_u16(&mut self, item: u16, buf: &mut bytes::BytesMut) -> Result<(), Self::Error> {
            buf.put_u16_le(item);
            Ok(())
        }

        fn encode_i32(&mut self, item: i32, buf: &mut bytes::BytesMut) -> Result<(), Self::Error> {
            buf.put_i32_le(item);
            Ok(())
        }

        fn encode_u32(&mut self, item: u32, buf: &mut bytes::BytesMut) -> Result<(), Self::Error> {
            buf.put_u32_le(item);
            Ok(())
        }

        fn encode_i64(&mut self, item: i64, buf: &mut bytes::BytesMut) -> Result<(), Self::Error> {
            buf.put_i64_le(item);
            Ok(())
        }

        fn encode_u64(&mut self, item: u64, buf: &mut bytes::BytesMut) -> Result<(), Self::Error> {
            buf.put_u64_le(item);
            Ok(())
        }

        fn encode_f32(&mut self, item: f32, buf: &mut bytes::BytesMut) -> Result<(), Self::Error> {
            buf.put_f32_le(item);
            Ok(())
        }

        fn encode_f64(&mut self, item: f64, buf: &mut bytes::BytesMut) -> Result<(), Self::Error> {
            buf.put_f64_le(item);
            Ok(())
        }

        fn encode_i128(
            &mut self,
            item: i128,
            buf: &mut bytes::BytesMut,
        ) -> Result<(), Self::Error> {
            buf.put_i128_le(item);
            Ok(())
        }

        fn encode_u128(
            &mut self,
            item: u128,
            buf: &mut bytes::BytesMut,
        ) -> Result<(), Self::Error> {
            buf.put_u128_le(item);
            Ok(())
        }
    }
}
