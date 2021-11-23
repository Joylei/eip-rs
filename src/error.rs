// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use bytes::Bytes;

use crate::{
    cip::{MessageReply, Status},
    eip::EipError,
};
use std::{error, fmt, io, net::AddrParseError, str::Utf8Error};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Utf8(Utf8Error),
    /// error with CIP status
    Cip(Status),
    /// error with EIP status
    Eip(EipError),
    /// Invalid Socket Address
    InvalidAddr(AddrParseError),
    InvalidCommandReply {
        /// expected command code
        expect: u16,
        /// actual command code returned
        actual: u16,
    },
    MessageRequestError(MessageReply<Bytes>),
}

impl error::Error for Error {
    #[inline(always)]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Utf8(e) => e.source(),
            Self::Io(e) => e.source(),
            Self::InvalidAddr(e) => e.source(),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Utf8(e) => {
                write!(f, "utf8 error: {}", e)?;
            }
            Self::Io(e) => {
                write!(f, "IO error: {}", e)?;
            }
            Self::Cip(e) => {
                write!(f, "CIP error: {}", e)?;
            }
            Self::Eip(e) => {
                write!(f, "EIP reply error: {}", e)?;
            }
            Self::InvalidAddr(e) => {
                write!(f, "invalid IP address: {}", e)?;
            }
            Self::InvalidCommandReply { expect, actual } => {
                write!(
                    f,
                    "invalid EIP command reply, expected command code: {}, actual command code: {}",
                    expect, actual
                )?;
            }
            Self::MessageRequestError(reply) => {
                write!(
                    f,
                    "Message request error: service {}, status {}",
                    reply.reply_service, reply.status
                )?;
            }
        }
        Ok(())
    }
}

impl From<Utf8Error> for Error {
    #[inline(always)]
    fn from(e: Utf8Error) -> Self {
        Self::Utf8(e)
    }
}

impl From<io::Error> for Error {
    #[inline(always)]
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<EipError> for Error {
    #[inline(always)]
    fn from(e: EipError) -> Self {
        Self::Eip(e)
    }
}

impl From<AddrParseError> for Error {
    #[inline(always)]
    fn from(e: AddrParseError) -> Self {
        Self::InvalidAddr(e)
    }
}
