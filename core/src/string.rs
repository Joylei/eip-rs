pub use alloc::string::{FromUtf8Error, String as StdString};
#[cfg(not(feature = "feat-inlinable-string"))]
pub use alloc::string::{String, String as StringExt};
#[cfg(feature = "feat-inlinable-string")]
pub use inlinable_string::{InlinableString, InlinableString as String, StringExt};
