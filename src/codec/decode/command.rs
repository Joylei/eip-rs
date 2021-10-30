// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{
    consts::{EIP_COMMAND_LIST_IDENTITY, EIP_COMMAND_LIST_SERVICE, EIP_COMMAND_REGISTER_SESSION},
    error::{EipError, Error},
    frame::{
        command_reply::{ListIdentityReply, ListServicesReply, RegisterSessionReply},
        CommonPacket, EncapsulationPacket,
    },
    objects::{identity::IdentityObject, service::ListServiceItem},
};
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use std::convert::TryFrom;

impl TryFrom<EncapsulationPacket<Bytes>> for RegisterSessionReply {
    type Error = Error;
    #[inline]
    fn try_from(src: EncapsulationPacket<Bytes>) -> Result<Self, Self::Error> {
        src.hdr.ensure_command(EIP_COMMAND_REGISTER_SESSION)?;
        let session_handle = src.hdr.session_handle;
        let reply_data = src.data;

        //RegisterSession
        if reply_data.len() < 4 {
            return Err(Error::Eip(EipError::InvalidData));
        }
        debug_assert_eq!(reply_data.len(), 4);
        //TODO: validate sender context
        let protocol_version = LittleEndian::read_u16(&reply_data[0..2]);
        debug_assert_eq!(protocol_version, 1);
        let session_options = LittleEndian::read_u16(&reply_data[2..4]);
        debug_assert_eq!(session_options, 0);
        Ok(Self {
            session_handle: session_handle,
            protocol_version,
        })
    }
}

impl TryFrom<EncapsulationPacket<Bytes>> for ListIdentityReply {
    type Error = Error;
    #[inline]
    fn try_from(src: EncapsulationPacket<Bytes>) -> Result<Self, Self::Error> {
        src.hdr.ensure_command(EIP_COMMAND_LIST_IDENTITY)?;
        let cpf = CommonPacket::try_from(src.data)?;
        if cpf.len() != 1 {
            return Err(Error::Eip(EipError::InvalidData));
        }
        // ListIdentity
        let res: Result<Vec<_>, _> = cpf
            .into_vec()
            .into_iter()
            .map(|item| {
                item.ensure_type_code(0x0C)?;
                IdentityObject::try_from(item.data)
            })
            .collect();
        Ok(Self(res?))
    }
}

impl TryFrom<EncapsulationPacket<Bytes>> for ListServicesReply {
    type Error = Error;
    #[inline]
    fn try_from(src: EncapsulationPacket<Bytes>) -> Result<Self, Self::Error> {
        src.hdr.ensure_command(EIP_COMMAND_LIST_SERVICE)?;
        let cpf = CommonPacket::try_from(src.data)?;
        if cpf.len() == 0 {
            log::debug!("expected at least 1 common packet item");
            return Err(Error::Eip(EipError::InvalidData));
        }
        // ListServices
        let res: Result<Vec<_>, _> = cpf
            .into_vec()
            .into_iter()
            .map(|item| {
                item.ensure_type_code(0x0C)?;
                ListServiceItem::try_from(item.data)
            })
            .collect();
        Ok(Self(res?))
    }
}
