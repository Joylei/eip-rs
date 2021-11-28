// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{
    connection::{
        ForwardCloseReply, ForwardCloseSuccess, ForwardOpenReply, ForwardOpenSuccess,
        ForwardRequestFail,
    },
    *,
};
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use core::convert::TryFrom;
use rseip_core::{cip::CommonPacketIterator, InnerError};
use std::io;

impl TryFrom<CommonPacketIterator> for ForwardOpenReply {
    type Error = Error;

    fn try_from(mut cpf: CommonPacketIterator) -> Result<Self> {
        if cpf.len() != 2 {
            return Err(Error::from(InnerError::InvalidData)
                .with_context("CIP - failed to decode reply of forward open"));
        }
        // should be null address
        cpf.next().unwrap()?.ensure_type_code(0)?;
        let data_item = cpf.next().unwrap()?;
        // should be unconnected data item
        data_item.ensure_type_code(0xB2)?;
        let mr_reply = MessageReply::try_from(data_item.data)?;
        if mr_reply.reply_service != 0xD4 && mr_reply.reply_service != 0xDB {
            return Err(Error::from(InnerError::InvalidData)
                .with_context("CIP - failed to decode reply of forward open"));
        }
        if mr_reply.status.is_ok() {
            let buf: Bytes = mr_reply.data;
            if buf.len() < 26 {
                return Err(Error::from(InnerError::InvalidData)
                    .with_context("CIP - failed to decode reply of forward open"));
            }
            let mut reply = ForwardOpenSuccess::default();
            reply.o_t_connection_id = LittleEndian::read_u32(&buf[0..4]);
            reply.t_o_connection_id = LittleEndian::read_u32(&buf[4..8]);
            reply.connection_serial_number = LittleEndian::read_u16(&buf[8..10]);
            reply.originator_vendor_id = LittleEndian::read_u16(&buf[10..12]);
            reply.originator_serial_number = LittleEndian::read_u32(&buf[12..16]);
            reply.o_t_api = LittleEndian::read_u32(&buf[16..20]);
            reply.t_o_api = LittleEndian::read_u32(&buf[20..24]);
            // buf[24], size in words
            let app_data_size = 2 * buf[24] as usize;
            if buf.len() != 26 + app_data_size {
                return Err(Error::from(InnerError::InvalidData)
                    .with_context("CIP - failed to decode reply of forward open"));
            }
            // reserved = buf[25]
            let app_data = buf.slice(26..);
            assert_eq!(app_data.len(), app_data_size);
            reply.app_data = app_data;
            Ok(ForwardOpenReply::Success {
                service_code: mr_reply.reply_service,
                reply,
            })
        } else {
            let buf: Bytes = mr_reply.data;
            let is_routing_error = mr_reply.status.is_routing_error();
            let data = parse_forward_request_fail(buf, is_routing_error)?;
            Ok(ForwardOpenReply::Fail(data))
        }
    }
}

impl TryFrom<CommonPacketIterator> for ForwardCloseReply {
    type Error = Error;
    fn try_from(mut cpf: CommonPacketIterator) -> Result<Self> {
        if cpf.len() != 2 {
            return Err(Error::from(InnerError::InvalidData)
                .with_context("CIP - failed to decode reply of forward close"));
        }
        // should be null address
        cpf.next().unwrap()?.ensure_type_code(0)?;
        let data_item = cpf.next().unwrap()?;
        // should be unconnected data item
        data_item.ensure_type_code(0xB2)?;
        let mr_reply = MessageReply::try_from(data_item.data)?;
        if mr_reply.reply_service != 0xCE {
            return Err(Error::from(InnerError::InvalidData)
                .with_context("CIP - failed to decode reply of forward close"));
        }
        if mr_reply.status.is_ok() {
            let buf: Bytes = mr_reply.data;
            if buf.len() < 10 {
                return Err(Error::from(InnerError::InvalidData)
                    .with_context("CIP - failed to decode reply of forward close"));
            }
            let mut reply = ForwardCloseSuccess::default();
            reply.connection_serial_number = LittleEndian::read_u16(&buf[0..2]);
            reply.originator_vendor_id = LittleEndian::read_u16(&buf[2..4]);
            reply.originator_serial_number = LittleEndian::read_u32(&buf[4..8]);

            // buf[8], size in words
            let app_data_size = 2 * buf[8] as usize;
            if buf.len() != 10 + app_data_size {
                return Err(Error::from(InnerError::InvalidData)
                    .with_context("CIP - failed to decode reply of forward close"));
            }
            // reserved = buf[9]
            let app_data = buf.slice(10..);
            assert_eq!(app_data.len(), app_data_size);
            reply.app_data = app_data;
            Ok(ForwardCloseReply::Success(reply))
        } else {
            let buf: Bytes = mr_reply.data;
            let is_routing_error = mr_reply.status.is_routing_error();
            let data = parse_forward_request_fail(buf, is_routing_error)?;
            Ok(ForwardCloseReply::Fail(data))
        }
    }
}

fn parse_forward_request_fail(buf: Bytes, is_routing_error: bool) -> Result<ForwardRequestFail> {
    let max_size = if is_routing_error { 9 } else { 8 };
    if buf.len() != max_size {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let res = ForwardRequestFail {
        connection_serial_number: LittleEndian::read_u16(&buf[0..2]),
        originator_vendor_id: LittleEndian::read_u16(&buf[2..4]),
        originator_serial_number: LittleEndian::read_u32(&buf[4..8]),
        remaining_path_size: if is_routing_error { Some(buf[8]) } else { None },
    };
    Ok(res)
}
