// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use rseip_cip::CipError;
use rseip_core::{Error, String};
use rseip_eip::ErrorStatus;
use std::{
    error, fmt, io,
    net::AddrParseError,
    ops::{Deref, DerefMut},
    str::Utf8Error,
};

use crate::client::ab_eip::PathError;

#[inline]
pub(crate) fn invalid_data(context: impl Into<String>) -> ClientError {
    Error::<InnerError>::from_invalid_data()
        .with_context(context)
        .into()
}

/// client error
#[derive(Debug)]
pub struct ClientError(Error<InnerError>);

impl Deref for ClientError {
    type Target = Error<InnerError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ClientError {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub enum InnerError {
    /// CIP error
    Cip(CipError),
    /// EIP error
    Eip(ErrorStatus),
    /// Invalid Socket Address
    InvalidAddr(AddrParseError),
    PathError(PathError),
}

impl From<Error<InnerError>> for ClientError {
    fn from(e: Error<InnerError>) -> Self {
        Self(e)
    }
}

impl From<Error<CipError>> for ClientError {
    fn from(e: Error<CipError>) -> Self {
        let e = e.map_err(InnerError::Cip);
        Self(e)
    }
}

impl From<CipError> for ClientError {
    fn from(e: CipError) -> Self {
        Self(Error::from_other(InnerError::Cip(e)))
    }
}

impl From<Error<ErrorStatus>> for ClientError {
    fn from(e: Error<ErrorStatus>) -> Self {
        let e = e.map_err(InnerError::Eip);
        Self(e)
    }
}

impl From<ErrorStatus> for ClientError {
    fn from(e: ErrorStatus) -> Self {
        Self(Error::from_other(InnerError::Eip(e)))
    }
}

impl From<Utf8Error> for ClientError {
    #[inline(always)]
    fn from(e: Utf8Error) -> Self {
        Self(e.into())
    }
}

impl From<io::Error> for ClientError {
    #[inline]
    fn from(e: io::Error) -> Self {
        Self(e.into())
    }
}

impl From<AddrParseError> for ClientError {
    #[inline]
    fn from(e: AddrParseError) -> Self {
        Self(Error::from_other(InnerError::InvalidAddr(e)))
    }
}

impl From<PathError> for ClientError {
    #[inline]
    fn from(e: PathError) -> Self {
        Self(Error::from_other(InnerError::PathError(e)))
    }
}

impl error::Error for ClientError {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.0.source()
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for InnerError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cip(e) => {
                write!(f, "{}", e)
            }
            Self::Eip(e) => {
                write!(f, "{}", e)
            }
            Self::InvalidAddr(e) => {
                write!(f, "invalid IP address: {}", e)
            }
            Self::PathError(e) => {
                write!(f, "{}", e)
            }
        }
    }
}

impl error::Error for InnerError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Cip(e) => e.source(),
            Self::Eip(e) => e.source(),
            Self::InvalidAddr(e) => Some(e),
            _ => None,
        }
    }
}
