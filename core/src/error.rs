// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use core::fmt;
pub use std::error::Error as StdError;
use std::io;

pub trait Error: Sized + StdError + From<io::Error> {
    fn with_kind(self, kind: &'static str) -> Self;

    /// Raised when there is general error when decoding a type.
    fn custom<T: fmt::Display>(msg: T) -> Self;

    /// Raised when receives a type different from what it was expecting.
    fn invalid_type<U: fmt::Display, E: fmt::Display>(unexp: U, exp: E) -> Self {
        Self::custom(format_args!("invalid type: {}, expected {}", unexp, exp))
    }

    /// Raised when receives a value of the right type but that
    /// is wrong for some other reason.
    fn invalid_value<U: fmt::Display, E: fmt::Display>(unexp: U, exp: E) -> Self {
        Self::custom(format_args!("invalid value: {}, expected {}", unexp, exp))
    }

    /// Raised when the input data contains too many
    /// or too few elements.
    fn invalid_length<E: fmt::Display>(len: usize, exp: E) -> Self {
        Self::custom(format_args!("invalid length: {}, expected {}", len, exp))
    }
}
