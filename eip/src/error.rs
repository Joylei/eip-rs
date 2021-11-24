// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::*;
use core::fmt;

impl From<ErrorStatus> for Error {
    #[inline]
    fn from(e: ErrorStatus) -> Self {
        Self::from_other(e)
    }
}

/// EIP command error status code
#[derive(Debug, Clone)]
pub enum ErrorStatus {
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

impl ErrorStatus {
    #[inline]
    pub fn status(&self) -> u16 {
        match self {
            Self::InvalidCommand => 0x0001,
            Self::InsufficientMemory => 0x0002,
            Self::InvalidData => 0x0003,
            Self::InvalidSession => 0x0064,
            Self::InvalidLength => 0x0065,
            Self::UnsupportedRevision => 0x0069,
            Self::Unknown(v) => *v,
        }
    }
}

impl std::error::Error for ErrorStatus {}

impl ErrorStatus {
    /// from error status
    #[inline]
    pub fn from_status(status: u16) -> StdResult<(), Self> {
        let status = match status {
            0x0000 => return Ok(()),
            0x0001 => Self::InvalidCommand,
            0x0002 => Self::InsufficientMemory,
            0x0003 => Self::InvalidData,
            0x0064 => Self::InvalidSession,
            0x0065 => Self::InvalidLength,
            0x0069 => Self::UnsupportedRevision,
            _ => Self::Unknown(status),
        };
        Err(status)
    }
}

impl fmt::Display for ErrorStatus {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "error-explain")]
        match self {
            Self::InvalidCommand => write!(f, "The sender issued an invalid or unsupported encapsulation command"),
            Self::InsufficientMemory => write!(f, "Insufficient memory resources in the receiver to handle the command"),
            Self::InvalidData => write!(f, "Poorly formed or incorrect data in the data portion of the encapsulation message"),
            Self::InvalidSession => write!(f, "An originator used an invalid session handle when sending an encapsulation message to the target"),
            Self::InvalidLength => write!(f, "The target received a message of invalid length"),
            Self::UnsupportedRevision => write!(f, "Unsupported encapsulation protocol revision"),
            Self::Unknown(v) => write!(f, "Unknown command error: {:#0x}", v),
        }

        #[cfg(not(feature = "error-explain"))]
        write!(f, "ENIP command error status: {:#0x}", self.status())
    }
}
