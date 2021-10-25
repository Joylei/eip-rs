use crate::{
    error::{Error, ResponseError},
    frame::{
        cip::{MessageRouterReply, UnconnectedSendReply},
        CommonPacketFormat, EncapsulationPacket,
    },
};
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use std::convert::TryFrom;
use std::io;

impl TryFrom<EncapsulationPacket<Bytes>> for UnconnectedSendReply<Bytes> {
    type Error = Error;
    #[inline]
    fn try_from(src: EncapsulationPacket<Bytes>) -> Result<Self, Self::Error> {
        if src.hdr.command != 0x6F {
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
            return Err(Error::Response(ResponseError::InvalidData));
        }
        // should be null address
        if !cpf[0].is_null_addr() {
            return Err(Error::Response(ResponseError::InvalidData));
        }
        let data_item = cpf.remove(1);
        // should be unconnected data item
        if data_item.type_code != 0xB2 {
            return Err(Error::Response(ResponseError::InvalidData));
        }
        let mr_reply = MessageRouterReply::try_from(data_item.data.unwrap())?;
        Ok(Self(mr_reply))
    }
}
