// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{
    consts::EIP_COMMAND_SEND_UNIT_DATA,
    error::{EipError, Error},
    frame::{
        cip::{ConnectedSendReply, MessageRouterReply},
        CommonPacket, EncapsulationPacket,
    },
};
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use std::convert::TryFrom;

impl TryFrom<EncapsulationPacket<Bytes>> for ConnectedSendReply<Bytes> {
    type Error = Error;
    #[inline]
    fn try_from(src: EncapsulationPacket<Bytes>) -> Result<Self, Self::Error> {
        src.hdr.ensure_command(EIP_COMMAND_SEND_UNIT_DATA)?;

        let interface_handle = LittleEndian::read_u32(&src.data[0..4]); // interface handle
        debug_assert_eq!(interface_handle, 0);
        // timeout = &src.data[4..6]

        let mut cpf = CommonPacket::try_from(src.data.slice(6..))?.into_vec();
        if cpf.len() != 2 {
            return Err(Error::Eip(EipError::InvalidData));
        }
        // should be connected address
        cpf[0].ensure_type_code(0xA1)?;
        let data_item = cpf.remove(1);
        // should be connected data item
        data_item.ensure_type_code(0xB1)?;
        if data_item.data.len() < 2 {
            return Err(Error::Eip(EipError::InvalidData));
        }
        //TODO: validate sequence count
        let _sequence_count = LittleEndian::read_u16(&data_item.data[0..2]);
        let mr_reply = MessageRouterReply::try_from(data_item.data.slice(2..))?;
        Ok(Self(mr_reply))
    }
}
