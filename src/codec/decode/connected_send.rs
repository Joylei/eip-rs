use crate::{
    error::{EipError, Error},
    frame::{
        cip::{ConnectedSendReply, MessageRouterReply},
        CommonPacketFormat, EncapsulationPacket,
    },
};
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use std::convert::TryFrom;
use std::io;

impl TryFrom<EncapsulationPacket<Bytes>> for ConnectedSendReply<Bytes> {
    type Error = Error;
    #[inline]
    fn try_from(src: EncapsulationPacket<Bytes>) -> Result<Self, Self::Error> {
        if src.hdr.command != 0x0070 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "SendRRData: unexpected reply command",
            )
            .into());
        }
        let interface_handle = LittleEndian::read_u32(&src.data[0..4]); // interface handle
        debug_assert_eq!(interface_handle, 0);
        // timeout = &src.data[4..6]

        //TODO: verify buf length
        let mut cpf = CommonPacketFormat::try_from(src.data.slice(6..))?.into_vec();
        if cpf.len() != 2 {
            return Err(Error::Eip(EipError::InvalidData));
        }
        // should be connected address
        if cpf[0].type_code != 0xA1 {
            return Err(Error::Eip(EipError::InvalidData));
        }
        let data_item = cpf.remove(1);
        // should be connected data item
        if data_item.type_code != 0xB1 {
            return Err(Error::Eip(EipError::InvalidData));
        }
        let mr_reply = MessageRouterReply::try_from(data_item.data.unwrap())?;
        Ok(Self(mr_reply))
    }
}
