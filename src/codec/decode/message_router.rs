// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{
    error::{EipError, Error},
    frame::cip::{MessageRouterReply, Status},
};
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use std::convert::TryFrom;

impl TryFrom<Bytes> for MessageRouterReply<Bytes> {
    type Error = Error;

    #[inline]
    fn try_from(buf: Bytes) -> Result<Self, Self::Error> {
        if buf.len() < 4 {
            return Err(Error::Eip(EipError::InvalidData));
        }
        let reply_service = buf[0];
        //reserved buf[1]
        let general_status = buf[2];
        let extended_status_size = buf[3];
        if buf.len() < 4 + extended_status_size as usize {
            return Err(Error::Eip(EipError::InvalidData));
        }
        let extended_status = match extended_status_size {
            0 => 0,
            1 => buf[4] as u16,
            2 => LittleEndian::read_u16(&buf[4..6]),
            _ => return Err(Error::Eip(EipError::InvalidData)),
        };
        let status = Status {
            general: general_status,
            extended: extended_status,
        };
        // //TODO: raise Error here?
        // if general_status != 0 {
        //     return Err(Error::CIPError(status));
        // }
        let pos = 4 + extended_status_size as usize;
        if status.is_routing_error() {
            if buf.len() != pos + 1 {
                return Err(Error::Eip(EipError::InvalidData));
            }
            let data = Self {
                reply_service,
                status,
                remaining_path_size: Some(buf[pos]),
                data: Bytes::new(),
            };
            Ok(data)
        } else {
            let data = buf.slice(pos..);
            Ok(Self::new(reply_service, status, data))
        }
    }
}
