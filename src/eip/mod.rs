// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

pub mod command;
pub mod common_packet;
pub(crate) mod context;
pub mod encapsulation;
mod framed;

pub use common_packet::{CommonPacket, CommonPacketItem};
pub use encapsulation::{EncapsulationHeader, EncapsulationPacket};
use std::fmt;

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
