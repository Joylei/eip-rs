// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::*;
use core::fmt;
use core::str::Utf8Error;
use std::io;

#[derive(Debug)]
pub struct Error<E> {
    e: InnerError<E>,
    context: Option<String>,
}

impl<E> Error<E> {
    #[inline]
    pub fn new(e: InnerError<E>) -> Self {
        Self { e, context: None }
    }

    #[inline]
    pub fn inner(&self) -> &InnerError<E> {
        &self.e
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut InnerError<E> {
        &mut self.e
    }

    #[inline]
    pub fn context(&self) -> Option<&str> {
        self.context.as_ref().map(|v| v.as_ref())
    }

    #[inline]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    #[inline]
    pub fn map_err<R>(self, f: impl Fn(E) -> R) -> Error<R> {
        let e = match self.e {
            InnerError::Io(e) => InnerError::Io(e),
            InnerError::Utf8(e) => InnerError::Utf8(e),
            InnerError::InvalidData => InnerError::InvalidData,
            InnerError::Other(e) => InnerError::Other(f(e)),
        };
        Error {
            e,
            context: self.context,
        }
    }

    #[inline]
    pub fn from_io(e: io::Error) -> Self {
        Self {
            e: InnerError::Io(e),
            context: None,
        }
    }

    #[inline]
    pub fn from_invalid_data() -> Self {
        Self {
            e: InnerError::InvalidData,
            context: None,
        }
    }

    #[inline]
    pub fn from_other(e: E) -> Self {
        Self {
            e: InnerError::Other(e),
            context: None,
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.e {
            InnerError::Io(ref e) => Some(e),
            InnerError::Utf8(ref e) => Some(e),
            InnerError::InvalidData => None,
            InnerError::Other(ref e) => Some(e),
        }
    }
}

impl<E> From<InnerError<E>> for Error<E> {
    #[inline]
    fn from(e: InnerError<E>) -> Self {
        Self { e, context: None }
    }
}

impl<E> From<io::Error> for Error<E> {
    #[inline]
    fn from(e: io::Error) -> Self {
        Self::from_io(e)
    }
}

impl<E> From<io::ErrorKind> for Error<E> {
    #[inline]
    fn from(e: io::ErrorKind) -> Self {
        Self::from_io(e.into())
    }
}

impl<E> From<Utf8Error> for Error<E> {
    #[inline]
    fn from(e: Utf8Error) -> Self {
        Self::new(InnerError::Utf8(e))
    }
}

impl<E: fmt::Display> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(context) = self.context() {
            write!(f, "{}\n:", context)?;
        }
        write!(f, "{}", self.e)
    }
}

#[derive(Debug)]
pub enum InnerError<E> {
    Io(io::Error),
    Utf8(Utf8Error),
    InvalidData,
    Other(E),
}

impl<E> From<io::Error> for InnerError<E> {
    #[inline]
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl<E> From<Utf8Error> for InnerError<E> {
    #[inline]
    fn from(e: Utf8Error) -> Self {
        InnerError::Utf8(e)
    }
}

impl<E: fmt::Display> fmt::Display for InnerError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{}", e),
            Self::Utf8(e) => write!(f, "{}", e),
            Self::InvalidData => write!(f, "invalid data"),
            Self::Other(e) => write!(f, "{}", e),
        }
    }
}
