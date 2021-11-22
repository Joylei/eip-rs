// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod connection;
mod message_router;

use super::ClientCodec;
use crate::{
    cip::{identity::IdentityObject, socket::SocketAddr, ListServiceItem, Revision},
    consts::{COMMON_PACKET_MAX_ITEM_COUNT, ENCAPSULATION_HEADER_LEN},
    eip::{
        common_packet::{CommonPacket, CommonPacketItem},
        encapsulation::EncapsulationHeader,
        EipError, EncapsulationPacket,
    },
    error::Error,
};
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use bytes::{Bytes, BytesMut};
use std::convert::TryFrom;
use tokio_util::codec::Decoder;

impl Decoder for ClientCodec {
    type Item = EncapsulationPacket<Bytes>;
    type Error = Error;

    #[inline]
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < ENCAPSULATION_HEADER_LEN {
            return Ok(None);
        }
        let data_len = LittleEndian::read_u16(&src[2..4]) as usize;
        //verify data length
        if ENCAPSULATION_HEADER_LEN + data_len > u16::MAX as usize {
            return Err(Error::Eip(EipError::InvalidLength));
        }
        if src.len() < ENCAPSULATION_HEADER_LEN + data_len {
            return Ok(None);
        }
        if src.len() > ENCAPSULATION_HEADER_LEN + data_len {
            // should no remaining buffer
            return Err(Error::Eip(EipError::InvalidLength));
        }
        let header_data = src.split_to(ENCAPSULATION_HEADER_LEN).freeze();
        let reply_data = src.split_to(data_len).freeze();
        let hdr = EncapsulationHeader::try_from(header_data)?;
        match hdr.status {
            0 => {}
            v if v > u16::MAX as u32 => {
                log::debug!("eip error - invalid status code: {}", v);
                return Err(Error::Eip(EipError::InvalidData));
            }
            v => return Err(Error::Eip(EipError::from(v as u16))),
        }
        return Ok(Some(EncapsulationPacket {
            hdr,
            data: reply_data,
        }));
    }
}

impl TryFrom<Bytes> for CommonPacket {
    type Error = Error;
    #[inline]
    fn try_from(mut buf: Bytes) -> Result<Self, Self::Error> {
        if buf.len() < 2 {
            return Err(Error::Eip(EipError::InvalidData));
        }
        let item_count = LittleEndian::read_u16(&buf[0..2]);
        if item_count > COMMON_PACKET_MAX_ITEM_COUNT {
            return Err(Error::Eip(EipError::InvalidData));
        }

        buf = buf.slice(2..);
        let mut items = Vec::new();
        for _ in 0..item_count {
            if buf.len() < 4 {
                return Err(Error::Eip(EipError::InvalidData));
            }
            let type_code = LittleEndian::read_u16(&buf[0..2]);
            let item_length = LittleEndian::read_u16(&buf[2..4]) as usize;
            if buf.len() < 4 + item_length {
                return Err(Error::Eip(EipError::InvalidData));
            }
            let item_data = buf.slice(4..4 + item_length);
            items.push(CommonPacketItem {
                type_code,
                data: item_data,
            });
            buf = buf.slice(4 + item_length..);
        }

        // should no remaining left
        if buf.len() != 0 {
            return Err(Error::Eip(EipError::InvalidData));
        }
        Ok(CommonPacket::from(items))
    }
}

impl TryFrom<Bytes> for IdentityObject {
    type Error = Error;
    #[inline]
    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        // dynamic size, so check size
        if data.len() < 33 {
            return Err(Error::Eip(EipError::InvalidData));
        }
        let product_name_len = data[32];
        //eprintln!("product_name_len: {}", product_name_len);

        if data.len() != 33 + product_name_len as usize + 1 {
            return Err(Error::Eip(EipError::InvalidData));
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
                .map_err(|e| e.utf8_error())?,
            state: *data.last().unwrap(),
        };

        Ok(identity)
    }
}

impl TryFrom<Bytes> for ListServiceItem {
    type Error = Error;
    #[inline(always)]
    fn try_from(buf: Bytes) -> Result<Self, Self::Error> {
        let mut item = ListServiceItem::default();
        item.protocol_version = LittleEndian::read_u16(&buf[0..2]);
        item.capability = LittleEndian::read_u16(&buf[2..4]);
        item.name = String::from_utf8(buf[4..20].to_vec()).map_err(|e| e.utf8_error())?;
        return Ok(item);
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

impl TryFrom<Bytes> for EncapsulationHeader {
    type Error = Error;
    #[inline(always)]
    fn try_from(buf: Bytes) -> Result<Self, Self::Error> {
        let mut hdr = EncapsulationHeader::default();
        hdr.command = LittleEndian::read_u16(&buf[0..2]);
        hdr.length = LittleEndian::read_u16(&buf[2..4]);
        hdr.session_handle = LittleEndian::read_u32(&buf[4..8]);
        hdr.status = LittleEndian::read_u32(&buf[8..12]);
        hdr.sender_context.copy_from_slice(&buf[12..20]);
        hdr.options = LittleEndian::read_u32(&buf[20..24]);
        Ok(hdr)
    }
}
