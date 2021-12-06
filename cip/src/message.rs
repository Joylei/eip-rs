// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::Status;
use crate::error::cip_error_reply;
use rseip_core::{codec::Encode, Error};

/// Message request
#[derive(Debug, Default, PartialEq, Eq)]
pub struct MessageRequest<P, D> {
    /// service request code
    pub service_code: u8,
    /// service request path
    pub path: P,
    /// service request data
    pub data: D,
}

impl<P, D> MessageRequest<P, D>
where
    P: Encode,
    D: Encode,
{
    #[inline]
    pub fn new(service_code: u8, path: P, data: D) -> Self {
        Self {
            service_code,
            path,
            data,
        }
    }
}

/// message router reply
#[derive(Debug)]
pub struct MessageReply<D> {
    /// reply service code
    pub reply_service: u8,
    /// general status and extended status
    pub status: Status,
    /// only present with routing type errors
    pub remaining_path_size: Option<u8>,
    pub data: D,
}

impl<D> MessageReply<D> {
    #[inline]
    pub fn new(reply_service: u8, status: Status, data: D) -> Self {
        Self {
            reply_service,
            status,
            remaining_path_size: None,
            data,
        }
    }

    #[inline]
    pub fn expect_service<E: Error>(&self, expected_service: u8) -> Result<(), E> {
        if self.reply_service != expected_service {
            Err(cip_error_reply(self.reply_service, expected_service))
        } else {
            Ok(())
        }
    }
}
