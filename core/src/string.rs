// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

pub use alloc::string::String as StdString;
#[cfg(not(feature = "feat-inlinable-string"))]
pub use alloc::string::{String, String as StringExt};
#[cfg(feature = "feat-inlinable-string")]
pub use inlinable_string::{InlinableString, InlinableString as String, StringExt};
