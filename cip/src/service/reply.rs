// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{Error, MessageReply, Result};
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use rseip_core::{cip::CommonPacketIter, InnerError};
use std::convert::TryFrom;

#[derive(Debug)]
pub struct ConnectedSendReply<D>(pub MessageReply<D>);

impl TryFrom<CommonPacketIter> for ConnectedSendReply<Bytes> {
    type Error = Error;
    #[inline]
    fn try_from(mut cpf: CommonPacketIter) -> Result<Self> {
        if cpf.len() != 2 {
            return Err(Error::from(InnerError::InvalidData)
                .with_context("common packet -  expected 2 items"));
        }
        // should be connected address
        cpf.next().unwrap()?.ensure_type_code(0xA1)?;
        let data_item = cpf.next().unwrap()?;
        // should be unconnected data item
        data_item.ensure_type_code(0xB1)?;
        if data_item.data.len() < 2 {
            return Err(Error::from(InnerError::InvalidData)
                .with_context("CIP - failed to decode message reply"));
        }

        //TODO: validate sequence count
        let _sequence_count = LittleEndian::read_u16(&data_item.data[0..2]);
        let mr_reply = MessageReply::try_from(data_item.data.slice(2..))?;
        Ok(Self(mr_reply))
    }
}

#[derive(Debug)]
pub struct UnconnectedSendReply<D>(pub MessageReply<D>);

impl TryFrom<CommonPacketIter> for UnconnectedSendReply<Bytes> {
    type Error = Error;
    #[inline]
    fn try_from(mut cpf: CommonPacketIter) -> Result<Self> {
        if cpf.len() != 2 {
            return Err(Error::from(InnerError::InvalidData)
                .with_context("common packet -  expected 2 items"));
        }
        // should be null address
        cpf.next().unwrap()?.ensure_type_code(0)?;
        let data_item = cpf.next().unwrap()?;
        // should be unconnected data item
        data_item.ensure_type_code(0xB2)?;
        let mr_reply = MessageReply::try_from(data_item.data)?;
        Ok(Self(mr_reply))
    }
}

#[derive(Debug)]
pub struct AttributeReply {
    pub id: u16,
    pub status: u16,
    pub data: Bytes,
}
