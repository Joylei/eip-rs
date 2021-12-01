// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod connection;
mod message_request;

use crate::{identity::IdentityObject, socket::SocketAddr, Error, ListServiceItem, Revision};
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use bytes::Bytes;
use core::convert::TryFrom;
use rseip_core::InnerError;

impl TryFrom<Bytes> for IdentityObject {
    type Error = Error;
    #[inline]
    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        // dynamic size, so check size
        if data.len() < 33 {
            return Err(Error::from(InnerError::InvalidData)
                .with_context("CIP - failed to decode identity object"));
        }
        let product_name_len = data[32];
        //eprintln!("product_name_len: {}", product_name_len);

        if data.len() != 33 + product_name_len as usize + 1 {
            return Err(Error::from(InnerError::InvalidData)
                .with_context("CIP - failed to decode identity object"));
        }
        let identity = IdentityObject {
            protocol_version: LittleEndian::read_u16(&data[..2]),
            socket_addr: SocketAddr::try_from(data.slice(2..18))?,
            vendor_id: LittleEndian::read_u16(&data[18..20]),
            device_type: LittleEndian::read_u16(&data[20..22]),
            product_code: LittleEndian::read_u16(&data[22..24]),
            revision: Revision {
                major: data[24],
                minor: data[25],
            },
            status: LittleEndian::read_u16(&data[26..28]),
            serial_number: LittleEndian::read_u32(&data[28..32]),
            product_name_len, //32
            product_name: String::from_utf8(data[33..33 + product_name_len as usize].to_vec())
                .map_err(|e| e.utf8_error())?
                .into(),
            state: *data.last().unwrap(),
        };

        Ok(identity)
    }
}

impl TryFrom<Bytes> for ListServiceItem {
    type Error = Error;
    #[inline(always)]
    fn try_from(buf: Bytes) -> Result<Self, Self::Error> {
        let item = ListServiceItem {
            protocol_version: LittleEndian::read_u16(&buf[0..2]),
            capability: LittleEndian::read_u16(&buf[2..4]),
            name: String::from_utf8(buf[4..20].to_vec())
                .map_err(|e| e.utf8_error())?
                .into(),
        };

        Ok(item)
    }
}

impl TryFrom<Bytes> for SocketAddr {
    type Error = Error;
    #[inline(always)]
    fn try_from(buf: Bytes) -> Result<Self, Self::Error> {
        let mut addr = SocketAddr {
            sin_family: BigEndian::read_i16(&buf[0..2]),
            sin_port: BigEndian::read_u16(&buf[2..4]),
            sin_addr: BigEndian::read_u32(&buf[4..8]),
            sin_zero: Default::default(),
        };
        addr.sin_zero.copy_from_slice(&buf[8..16]);
        Ok(addr)
    }
}
