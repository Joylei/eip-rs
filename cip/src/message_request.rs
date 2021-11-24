// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::Status;
use crate::codec::Encodable;

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
    P: Encodable,
    D: Encodable,
{
    #[inline(always)]
    pub fn new(service_code: u8, path: P, data: D) -> Self {
        Self {
            service_code,
            path,
            data,
        }
    }

    #[inline(always)]
    pub fn set_service_code(mut self, service_code: u8) -> Self {
        self.service_code = service_code;
        self
    }
    #[inline(always)]
    pub fn set_path(mut self, path: P) -> Self {
        self.path = path;
        self
    }
    #[inline(always)]
    pub fn set_data(mut self, data: D) -> Self {
        self.data = data;
        self
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
    #[inline(always)]
    pub fn new(reply_service: u8, status: Status, data: D) -> Self {
        Self {
            reply_service,
            status,
            remaining_path_size: None,
            data,
        }
    }

    #[inline(always)]
    pub fn set_reply_service(mut self, reply_service: u8) -> Self {
        self.reply_service = reply_service;
        self
    }

    #[inline(always)]
    pub fn set_status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    #[inline(always)]
    pub fn set_data(mut self, data: D) -> Self {
        self.data = data;
        self
    }
}
