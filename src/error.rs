use std::error;
use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Response(ResponseError),
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Io(e) => e.source(),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => {
                write!(f, "ENIP codec error: {}", e)?;
            }
            Error::Response(e) => {
                write!(f, "{}", e)?;
            }
        }
        Ok(())
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<ResponseError> for Error {
    fn from(e: ResponseError) -> Self {
        Self::Response(e)
    }
}

#[derive(Debug)]
pub enum ResponseError {
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

impl From<u16> for ResponseError {
    fn from(src: u16) -> Self {
        match src {
            0x0000 => unreachable!(), // represents success
            0x0001 => Self::InvalidCommand,
            0x0002 => Self::InsufficientMemory,
            0x0003 => Self::InvalidData,
            0x0064 => Self::InvalidSession,
            0x0065 => Self::InvalidLength,
            0x0069 => Self::UnsupportedRevision,
            _ => Self::Unknown(src),
        }
    }
}

impl fmt::Display for ResponseError {
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
