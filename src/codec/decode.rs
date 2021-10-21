use super::ClientCodec;
use crate::{
    consts::ENCAPSULATION_HEADER_LEN,
    error::{Error, ResponseError},
    frame::{
        common_packet::{CommonPacketFormat, CommonPacketItem},
        encapsulation::EncapsulationHeader,
        Response,
    },
    objects::{identity::IdentityObject, service::ListServiceItem, socket::SocketAddr},
};
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use bytes::Bytes;
use std::convert::TryFrom;
use tokio_util::codec::Decoder;

impl Decoder for ClientCodec {
    type Item = Response;
    type Error = Error;
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < ENCAPSULATION_HEADER_LEN {
            return Ok(None);
        }
        let data_len = LittleEndian::read_u16(&src[2..4]) as usize;
        //verify data length
        if ENCAPSULATION_HEADER_LEN + data_len > u16::MAX as usize {
            return Err(Error::Response(ResponseError::InvalidLength));
        }
        if src.len() < ENCAPSULATION_HEADER_LEN + data_len {
            return Ok(None);
        }
        let header_data = src.split_to(ENCAPSULATION_HEADER_LEN).freeze();
        let reply_data = src.split_to(data_len).freeze();
        let hdr = EncapsulationHeader::try_from(header_data)?;
        match hdr.status {
            0 => {}
            v => return Err(Error::Response(ResponseError::from(v as u16))),
        }
        match hdr.command {
            0x04 => {
                // ListServices
                //TODO: validate sender context
                let cpf = CommonPacketFormat::try_from(reply_data)?;
                if cpf.len() != 1 {
                    return Err(Error::Response(ResponseError::InvalidData));
                }
                let res: Result<Vec<_>, _> = cpf
                    .into_vec()
                    .into_iter()
                    .map(|item| {
                        if item.type_code != 0x100 {
                            return Err(Error::Response(ResponseError::InvalidData));
                        }
                        let item_data = item.data.unwrap();
                        if item_data.len() != 20 {
                            return Err(Error::Response(ResponseError::InvalidData));
                        }
                        ListServiceItem::try_from(item_data)
                    })
                    .collect();
                Ok(Some(Response::ListServices(res?)))
            }
            0x63 => {
                let cpf = CommonPacketFormat::try_from(reply_data)?;
                if cpf.len() != 1 {
                    return Err(Error::Response(ResponseError::InvalidData));
                }
                // ListIdentity
                let res: Result<Vec<_>, _> = cpf
                    .into_vec()
                    .into_iter()
                    .map(|item| {
                        if item.type_code != 0x0C {
                            return Err(Error::Response(ResponseError::InvalidData));
                        }
                        let item_data = item.data.unwrap();
                        IdentityObject::try_from(item_data)
                    })
                    .collect();
                Ok(Some(Response::ListIdentity(res?)))
            }
            0x64 => {
                // ListInterfaces
                let cpf = CommonPacketFormat::try_from(reply_data)?;
                if cpf.len() != 1 {
                    return Err(Error::Response(ResponseError::InvalidData));
                }
                todo!()
            }
            0x65 => {
                //RegisterSession
                if reply_data.len() < 4 {
                    return Err(Error::Response(ResponseError::InvalidData));
                }
                //TODO: validate sender context
                let protocol_version = LittleEndian::read_u16(&reply_data[0..2]);
                debug_assert_eq!(protocol_version, 1);
                let session_options = LittleEndian::read_u16(&reply_data[2..4]);
                debug_assert_eq!(session_options, 0);
                Ok(Some(Response::RegisterSession {
                    session_handle: hdr.session_handler,
                    protocol_version,
                }))
            }
            0x6F => {
                // SendRRData
                if reply_data.len() < 6 {
                    return Err(Error::Response(ResponseError::InvalidData));
                }
                let interface_handle = LittleEndian::read_u32(&reply_data[0..4]); // 0 for CIP
                debug_assert!(interface_handle == 0);
                let _timeout = LittleEndian::read_u16(&reply_data[4..6]);
                let cpf_data = reply_data.slice(6..);
                let cpf = CommonPacketFormat::try_from(cpf_data)?;
                todo!()
            }
            _ => {
                return Err(Error::Response(ResponseError::InvalidCommand));
            }
        }
    }
}

impl TryFrom<Bytes> for CommonPacketFormat {
    type Error = Error;
    fn try_from(mut buf: Bytes) -> Result<Self, Self::Error> {
        if buf.len() < 2 {
            return Err(Error::Response(ResponseError::InvalidData));
        }
        let item_count = LittleEndian::read_u16(&buf[0..2]);
        buf = buf.slice(2..);
        let mut items = Vec::with_capacity(item_count as usize);
        for _ in 0..item_count {
            if buf.len() < 4 {
                return Err(Error::Response(ResponseError::InvalidData));
            }
            let type_code = LittleEndian::read_u16(&buf[0..2]);
            let item_length = LittleEndian::read_u16(&buf[2..4]) as usize;
            if buf.len() < 4 + item_length {
                return Err(Error::Response(ResponseError::InvalidData));
            }
            let item_data = buf.slice(4..4 + item_length);
            items.push(CommonPacketItem {
                type_code,
                data: Some(item_data),
            });
            buf = buf.slice(4 + item_length..);
        }

        // no remaining left
        if buf.len() != 0 {
            return Err(Error::Response(ResponseError::InvalidData));
        }
        Ok(CommonPacketFormat::from(items))
    }
}

impl TryFrom<Bytes> for IdentityObject {
    type Error = Error;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        // dynamic size, so check size
        if data.len() < 33 {
            return Err(Error::Response(ResponseError::InvalidData));
        }
        let product_name_len = data[32];
        //eprintln!("product_name_len: {}", product_name_len);

        if data.len() != 33 + product_name_len as usize + 1 {
            return Err(Error::Response(ResponseError::InvalidData));
        }
        let identity = IdentityObject {
            protocol_version: LittleEndian::read_u16(&data[..2]),
            socket_addr: SocketAddr::try_from(data.slice(2..18))?,
            vendor_id: LittleEndian::read_u16(&data[18..20]),
            device_type: LittleEndian::read_u16(&data[20..22]),
            product_code: LittleEndian::read_u16(&data[22..24]),
            revision: [data[24], data[25]],
            status: LittleEndian::read_u16(&data[26..28]),
            serial_number: LittleEndian::read_u32(&data[28..32]),
            product_name_len, //32
            product_name: unsafe {
                //TODO: is it OK?
                String::from_utf8_unchecked(data[33..33 + product_name_len as usize].to_vec())
            },
            state: *data.last().unwrap(),
        };

        Ok(identity)
    }
}

impl TryFrom<Bytes> for ListServiceItem {
    type Error = Error;
    fn try_from(buf: Bytes) -> Result<Self, Self::Error> {
        let mut item = ListServiceItem::default();
        item.protocol_version = LittleEndian::read_u16(&buf[0..2]);
        item.capability = LittleEndian::read_u16(&buf[2..4]);
        item.name.copy_from_slice(&buf[4..20]);
        return Ok(item);
    }
}

impl TryFrom<Bytes> for SocketAddr {
    type Error = Error;
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
    fn try_from(buf: Bytes) -> Result<Self, Self::Error> {
        let mut hdr = EncapsulationHeader::default();
        hdr.command = LittleEndian::read_u16(&buf[0..2]);
        hdr.length = LittleEndian::read_u16(&buf[2..4]);
        hdr.session_handler = LittleEndian::read_u32(&buf[4..8]);
        hdr.status = LittleEndian::read_u32(&buf[8..12]);
        hdr.sender_context.copy_from_slice(&buf[12..20]);
        hdr.options = LittleEndian::read_u32(&buf[20..24]);
        Ok(hdr)
    }
}
