// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2020-2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::frame::cip::Status;
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

/// EIP error
#[derive(Debug)]
pub enum EipError {
    InvalidCommand,
    /// failed to request memory
    InsufficientMemory,
    /// invalid data in the data portion of the encapsulation message
    InvalidData,
    /// invalid session
    InvalidSession,
    /// invalid message length
    InvalidLength,
    /// unsupported encapsulation protocol revision
    UnsupportedRevision,
    Unknown(u16),
}

impl From<u16> for EipError {
    #[inline]
    fn from(status: u16) -> Self {
        match status {
            0x0000 => unreachable!(), // represents success
            0x0001 => Self::InvalidCommand,
            0x0002 => Self::InsufficientMemory,
            0x0003 => Self::InvalidData,
            0x0064 => Self::InvalidSession,
            0x0065 => Self::InvalidLength,
            0x0069 => Self::UnsupportedRevision,
            _ => Self::Unknown(status),
        }
    }
}

impl fmt::Display for EipError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCommand => write!(f, "The sender issued an invalid or unsupported encapsulation command"),
            Self::InsufficientMemory => write!(f, "Insufficient memory resources in the receiver to handle the command"),
            Self::InvalidData => write!(f, "Poorly formed or incorrect data in the data portion of the encapsulation message"),
            Self::InvalidSession => write!(f, "An originator used an invalid session handle when sending an encapsulation message to the target"),
            Self::InvalidLength => write!(f, "The target received a message of invalid length"),
            Self::UnsupportedRevision => write!(f, "Unsupported encapsulation protocol revision"),
            Self::Unknown(v) => write!(f, "Unknown command error: {:#0x}", v),
        }
    }
}
