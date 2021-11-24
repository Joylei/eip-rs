// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::*;
use bytes::Bytes;
use core::{convert::From, fmt};

#[derive(Debug)]
pub enum CipError {
    Cip(Status),
    MessageReplyError(MessageReply<Bytes>),
}

impl std::error::Error for CipError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Cip(e) => Some(e),
            _ => None,
        }
    }
}

impl From<Status> for CipError {
    #[inline]
    fn from(s: Status) -> Self {
        CipError::Cip(s)
    }
}

impl From<CipError> for Error {
    #[inline]
    fn from(e: CipError) -> Self {
        Self::from_other(e)
    }
}

impl From<Status> for Error {
    #[inline]
    fn from(s: Status) -> Self {
        Self::from_other(CipError::Cip(s))
    }
}

impl fmt::Display for CipError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cip(e) => {
                write!(f, "CIP error: {}", e)?;
            }
            Self::MessageReplyError(reply) => {
                write!(
                    f,
                    "Message Reply error: service {}, status {}",
                    reply.reply_service, reply.status
                )?;
            }
        }
        Ok(())
    }
}
