// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

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

pub use core::result::Result as StdResult;
pub use either::Either;
pub use error::{Error, StdError};
pub use string::*;
