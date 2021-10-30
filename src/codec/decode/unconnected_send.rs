// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{
    consts::EIP_COMMAND_SEND_RRDATA,
    error::{EipError, Error},
    frame::{
        cip::{MessageRouterReply, UnconnectedSendReply},
        CommonPacket, EncapsulationPacket,
    },
};
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use std::convert::TryFrom;

impl TryFrom<EncapsulationPacket<Bytes>> for UnconnectedSendReply<Bytes> {
    type Error = Error;
    #[inline]
    fn try_from(src: EncapsulationPacket<Bytes>) -> Result<Self, Self::Error> {
        src.hdr.ensure_command(EIP_COMMAND_SEND_RRDATA)?;
        let interface_handle = LittleEndian::read_u32(&src.data[0..4]); // interface handle
        debug_assert_eq!(interface_handle, 0);
        // timeout = &src.data[4..6]

        //TODO: verify buf length
        let mut cpf = CommonPacket::try_from(src.data.slice(6..))?.into_vec();
        if cpf.len() != 2 {
            return Err(Error::Eip(EipError::InvalidData));
        }
        // should be null address
        cpf[0].ensure_type_code(0)?;
        let data_item = cpf.remove(1);
        // should be unconnected data item
        data_item.ensure_type_code(0xB2)?;
        let mr_reply = MessageRouterReply::try_from(data_item.data)?;
        Ok(Self(mr_reply))
    }
}
