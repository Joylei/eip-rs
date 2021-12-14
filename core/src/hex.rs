// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use core::fmt::{self, Debug, LowerHex};

/// print hex
pub struct Hex<T> {
    inner: T,
    lower: bool,
    prefix: bool,
}

impl<T> Hex<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            lower: true,
            prefix: true,
        }
    }

    /// print lower hex
    pub fn lower(mut self, lower: bool) -> Self {
        self.lower = lower;
        self
    }

    /// print prefix `0x`
    pub fn prefix(mut self, prefix: bool) -> Self {
        self.prefix = prefix;
        self
    }
}

impl<T: Debug> fmt::Debug for Hex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.prefix {
            write!(f, "0x")?;
        }
        //Rust 1.26.0 and up
        if self.lower {
            write!(f, "{:02x?}", self.inner)
        } else {
            write!(f, "{:02X?}", self.inner)
        }
    }
}

impl<T: Debug> fmt::Display for Hex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

pub trait AsHex {
    fn as_hex(&self) -> Hex<&dyn Debug>;
}

impl<T: Debug + LowerHex> AsHex for T {
    fn as_hex(&self) -> Hex<&dyn Debug> {
        Hex::new(self)
    }
}
