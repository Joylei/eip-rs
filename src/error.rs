// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::client::ab_eip::PathError;
use core::fmt;
use rseip_core::{Error, String};
use std::io;

/// client error
#[derive(Debug)]
pub enum ClientError {
    Io { kind: &'static str, err: io::Error },
    Custom { kind: &'static str, msg: String },
}

impl ClientError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Io { kind, .. } => kind,
            Self::Custom { kind, .. } => kind,
        }
    }

    pub fn kind_mut(&mut self) -> &mut &'static str {
        match self {
            Self::Io { kind, .. } => kind,
            Self::Custom { kind, .. } => kind,
        }
    }
}

impl std::error::Error for ClientError {}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { kind, err } => write!(f, "{} - {}", kind, err),
            Self::Custom { kind, msg } => write!(f, "{} - {}", kind, msg),
        }
    }
}

impl Error for ClientError {
    fn with_kind(mut self, kind: &'static str) -> Self {
        *self.kind_mut() = kind;
        self
    }

    fn custom<T: core::fmt::Display>(msg: T) -> Self {
        Self::Custom {
            kind: "custom",
            msg: msg.to_string().into(),
        }
    }
}

impl From<io::Error> for ClientError {
    fn from(e: io::Error) -> Self {
        Self::Io { kind: "io", err: e }
    }
}

impl From<PathError> for ClientError {
    fn from(e: PathError) -> Self {
        Self::custom(e).with_kind("tag path error")
    }
}
