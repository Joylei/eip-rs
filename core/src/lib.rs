// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

//#![warn(missing_docs)]

/// alloc crate
#[allow(unused_imports)]
#[cfg_attr(feature = "no_std", macro_use)]
extern crate alloc;

pub extern crate smallvec;

#[cfg(feature = "cip")]
pub mod cip;
mod either;
mod error;
mod string;

pub use either::Either;
pub use error::{Error, InnerError};
pub use string::*;
