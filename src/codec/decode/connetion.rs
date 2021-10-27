use crate::{
    error::{Error, ResponseError},
    frame::{
        cip::{connection::ForwardOpenReply, MessageRouterReply},
        CommonPacketFormat, EncapsulationPacket,
    },
};
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use std::{convert::TryFrom, io};

impl TryFrom<EncapsulationPacket<Bytes>> for ForwardOpenReply {
    type Error = Error;

    fn try_from(src: EncapsulationPacket<Bytes>) -> Result<Self, Self::Error> {
        if src.hdr.command != 0xD4 {
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
        if mr_reply.reply_service != 0xD4 {
            return Err(Error::Response(ResponseError::InvalidData));
        }
        let buf: Bytes = mr_reply.data;
        if buf.len() < 26 {
            return Err(Error::Response(ResponseError::InvalidData));
        }
        let mut parameters = Self::default();
        parameters.o_t_connection_id = LittleEndian::read_u32(&buf[0..4]);
        parameters.t_o_connection_id = LittleEndian::read_u32(&buf[4..8]);
        parameters.connection_serial_number = LittleEndian::read_u16(&buf[8..10]);
        parameters.originator_vendor_id = LittleEndian::read_u16(&buf[10..12]);
        parameters.originator_serial_number = LittleEndian::read_u32(&buf[12..16]);
        parameters.o_t_api = LittleEndian::read_u32(&buf[16..20]);
        parameters.t_o_api = LittleEndian::read_u32(&buf[20..24]);
        // buf[24], size in words
        let app_data_size = 2 * buf[24] as usize;
        if buf.len() != 26 + app_data_size {
            return Err(Error::Response(ResponseError::InvalidData));
        }
        // reserved = buf[25]
        let app_data = buf.slice(26..);
        assert_eq!(app_data.len(), app_data_size);
        parameters.app_data = app_data;
        Ok(parameters)
    }
}
