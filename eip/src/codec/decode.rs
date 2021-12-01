// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use byteorder::{ByteOrder, LittleEndian};
use bytes::{Bytes, BytesMut};
use core::convert::TryFrom;
use rseip_core::InnerError;
use tokio_util::codec::Decoder;

use crate::{consts::*, EncapsulationHeader, EncapsulationPacket, Error, ErrorStatus};

use super::ClientCodec;

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
            return Err(Error::from(InnerError::InvalidData)
                .with_context("ENIP command reply: invalid packet - data too large"));
        }
        if src.len() < ENCAPSULATION_HEADER_LEN + data_len {
            return Ok(None);
        }
        if src.len() > ENCAPSULATION_HEADER_LEN + data_len {
            // should no remaining buffer
            return Err(Error::from(InnerError::InvalidData)
                .with_context("ENIP command reply: invalid packet"));
        }
        let header_data = src.split_to(ENCAPSULATION_HEADER_LEN).freeze();
        let reply_data = src.split_to(data_len).freeze();
        let hdr = EncapsulationHeader::try_from(header_data)?;
        match hdr.status {
            v if v > u16::MAX as u32 => {
                log::trace!("ENIP error status: {:#0x}", v);
                return Err(Error::from(InnerError::InvalidData)
                    .with_context("ENIP command reply: invalid packet"));
            }
            v => ErrorStatus::from_status(v as u16)?,
        }
        Ok(Some(EncapsulationPacket {
            hdr,
            data: reply_data,
        }))
    }
}

impl TryFrom<Bytes> for EncapsulationHeader {
    type Error = Error;
    #[inline(always)]
    fn try_from(buf: Bytes) -> Result<Self, Self::Error> {
        let mut hdr = EncapsulationHeader {
            command: LittleEndian::read_u16(&buf[0..2]),
            length: LittleEndian::read_u16(&buf[2..4]),
            session_handle: LittleEndian::read_u32(&buf[4..8]),
            status: LittleEndian::read_u32(&buf[8..12]),
            options: LittleEndian::read_u32(&buf[20..24]),
            ..Default::default()
        };
        hdr.sender_context.copy_from_slice(&buf[12..20]);

        Ok(hdr)
    }
}
