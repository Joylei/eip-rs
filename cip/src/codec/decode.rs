// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{
    connection::{
        ForwardCloseReply, ForwardCloseSuccess, ForwardOpenReply, ForwardOpenSuccess,
        ForwardRequestFail,
    },
    error::{cip_error, cip_error_status},
    identity::IdentityObject,
    socket::SocketAddr,
    *,
};
use bytes::Buf;
use rseip_core::{
    cip::CommonPacketIter,
    codec::{Decode, Decoder},
    hex::AsHex,
    Either, Error,
};

impl<'de> Decode<'de> for IdentityObject {
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: rseip_core::codec::Decoder<'de>,
    {
        // dynamic size, so check size
        decoder.ensure_size(33)?;
        //let product_name_len = data[32];

        let identity = IdentityObject {
            protocol_version: decoder.decode_u16(),
            socket_addr: {
                let addr = decoder.buf_mut().copy_to_bytes(16);
                SocketAddr::from_bytes::<D::Error>(addr)?
            },
            vendor_id: decoder.decode_u16(),
            device_type: decoder.decode_u16(),
            product_code: decoder.decode_u16(),
            revision: Revision {
                major: decoder.decode_u8(),
                minor: decoder.decode_u8(),
            },
            status: decoder.decode_u16(),
            serial_number: decoder.decode_u32(),
            product_name: {
                let name_len = decoder.decode_u8();
                decoder.ensure_size(name_len as usize + 1)?;
                let data = decoder.buf_mut().copy_to_bytes(name_len as usize).to_vec();
                String::from_utf8_lossy(&data).into_owned().into()
            },
            state: decoder.decode_u8(),
        };

        Ok(identity)
    }
}

impl<'de> Decode<'de> for ListServiceItem {
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        debug_assert!(decoder.remaining() > 4);

        let item = ListServiceItem {
            protocol_version: decoder.decode_u16(),
            capability: decoder.decode_u16(),
            name: {
                decoder.ensure_size(16)?;
                let data = decoder.buf_mut().copy_to_bytes(16);
                String::from_utf8_lossy(&data).into_owned()
            },
        };

        Ok(item)
    }
}

impl<'de, R> Decode<'de> for MessageReply<R>
where
    R: Decode<'de>,
{
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Self::decode_with(decoder, |mut d, _service, status| {
            if status.is_err() {
                Err(cip_error_status(status))
            } else {
                d.decode_any()
            }
        })
    }
}

impl<R> MessageReply<R> {
    pub fn decode_with<'de, D, F>(mut decoder: D, f: F) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
        F: Fn(D, u8, Status) -> Result<R, D::Error>,
    {
        decoder.ensure_size(4)?;
        let reply_service = decoder.decode_u8(); // buf[0]
        decoder.buf_mut().advance(1); // buf[1]
        let general_status = decoder.decode_u8(); //buf[2]
        let extended_status_size = decoder.decode_u8(); // buf[3]
        decoder.ensure_size(extended_status_size as usize)?;
        let extended_status = match extended_status_size {
            0 => None,
            1 => Some(decoder.decode_u8() as u16),
            2 => Some(decoder.decode_u16()),
            v => return Err(D::Error::invalid_value("one of 0,1,2", v)),
        };
        let status = Status {
            general: general_status,
            extended: extended_status,
        };

        let data = f(decoder, reply_service, status)?;
        Ok(MessageReply::new(reply_service, status, data))
    }

    #[inline]
    pub fn decode_connected_send<'de, D>(
        mut cpf: CommonPacketIter<'de, D>,
    ) -> Result<(u16, MessageReply<R>), D::Error>
    where
        D: Decoder<'de>,
        R: Decode<'de> + 'static,
    {
        if cpf.len() != 2 {
            return Err(cip_error("common packet - expected 2 items"));
        }
        // should be connected address
        cpf.next_item()
            .unwrap()?
            .ensure_type_code::<D::Error>(0xA1)?;
        let data_item: CommonPacketItem<(u16, _)> = cpf.next_typed().unwrap()?;
        // should be connected data item
        data_item.ensure_type_code::<D::Error>(0xB1)?;
        Ok(data_item.data)
    }

    #[inline]
    pub fn decode_unconnected_send<'de, D>(
        mut cpf: CommonPacketIter<'de, D>,
    ) -> Result<MessageReply<R>, D::Error>
    where
        D: Decoder<'de>,
        R: Decode<'de> + 'static,
    {
        if cpf.len() != 2 {
            return Err(cip_error("common packet - expected 2 items"));
        }
        // should be null address
        cpf.next_item().unwrap()?.ensure_type_code::<D::Error>(0)?;
        let data_item: CommonPacketItem<_> = cpf.next_typed().unwrap()?;

        // should be unconnected data item
        data_item.ensure_type_code::<D::Error>(0xB2)?;
        let reply = data_item.data;
        Ok(reply)
    }
}

impl MessageReply<ForwardOpenReply> {
    #[inline]
    pub fn decode_forward_open<'de, D>(
        mut cpf: CommonPacketIter<'de, D>,
    ) -> Result<MessageReply<ForwardOpenReply>, D::Error>
    where
        D: Decoder<'de>,
    {
        if cpf.len() != 2 {
            return Err(cip_error("common packet - expected 2 items"));
        }
        // should be null address
        cpf.next_item().unwrap()?.ensure_type_code::<D::Error>(0)?;
        // should be unconnected data item
        // let data_item: CommonPacketItem<_> = cpf.next_typed().unwrap()?;
        // data_item.ensure_type_code::<D::Error>(0xB2)?;
        let data_item = cpf
            .visit(|d, type_code| {
                if type_code != 0xB2 {
                    return Err(Error::invalid_value(
                        format_args!("common packet item type {}", type_code.as_hex()),
                        0xB2.as_hex(),
                    ));
                }
                MessageReply::decode_with(d, |d, _, status| {
                    if status.is_err() {
                        let v = decode_forward_fail(d, status)?;
                        Ok(Either::Right(v))
                    } else {
                        let v = decode_forward_open_success(d)?;
                        Ok(Either::Left(v))
                    }
                })
            })
            .unwrap()?;
        Ok(data_item.data)
    }
}

impl MessageReply<ForwardCloseReply> {
    #[inline]
    pub fn decode_forward_close<'de, D>(
        mut cpf: CommonPacketIter<'de, D>,
    ) -> Result<MessageReply<ForwardCloseReply>, D::Error>
    where
        D: Decoder<'de>,
    {
        if cpf.len() != 2 {
            return Err(cip_error("common packet - expected 2 items"));
        }
        // should be null address
        cpf.next_item().unwrap()?.ensure_type_code::<D::Error>(0)?;
        // should be unconnected data item
        // let data_item: CommonPacketItem<_> = cpf.next_typed().unwrap()?;
        // data_item.ensure_type_code::<D::Error>(0xB2)?;
        let data_item = cpf
            .visit(|d, type_code| {
                if type_code != 0xB2 {
                    return Err(Error::invalid_value(
                        format_args!("common packet item type {}", type_code.as_hex()),
                        0xB2.as_hex(),
                    ));
                }
                MessageReply::decode_with(d, |d, _, status| {
                    if status.is_err() {
                        let v = decode_forward_fail(d, status)?;
                        Ok(Either::Right(v))
                    } else {
                        let v = decode_forward_close_success(d)?;
                        Ok(Either::Left(v))
                    }
                })
            })
            .unwrap()?;
        Ok(data_item.data)
    }
}

fn decode_forward_open_success<'de, D>(mut decoder: D) -> Result<ForwardOpenSuccess, D::Error>
where
    D: Decoder<'de>,
{
    decoder.ensure_size(26)?;
    let v = ForwardOpenSuccess {
        o_t_connection_id: decoder.decode_u32(),
        t_o_connection_id: decoder.decode_u32(),
        connection_serial_number: decoder.decode_u16(),
        originator_vendor_id: decoder.decode_u16(),
        originator_serial_number: decoder.decode_u32(),
        o_t_api: decoder.decode_u32(),
        t_o_api: decoder.decode_u32(),
        app_data: {
            // buf[24], size in words
            let app_data_size = 2 * decoder.decode_u8() as usize;
            decoder.ensure_size(app_data_size)?;
            decoder.buf_mut().advance(1); // reserved = buf[25]
            decoder.buf_mut().copy_to_bytes(app_data_size)
        },
    };
    Ok(v)
}

fn decode_forward_close_success<'de, D>(mut decoder: D) -> Result<ForwardCloseSuccess, D::Error>
where
    D: Decoder<'de>,
{
    decoder.ensure_size(10)?;
    let v = ForwardCloseSuccess {
        connection_serial_number: decoder.decode_u16(), //  buf[0..2]
        originator_vendor_id: decoder.decode_u16(),     // buf[2..4]
        originator_serial_number: decoder.decode_u32(), // buf[4..8]
        app_data: {
            // buf[8], size in words
            let app_data_size = 2 * decoder.decode_u8() as usize;
            decoder.ensure_size(app_data_size)?;
            decoder.buf_mut().advance(1); // reserved = buf[9]
            decoder.buf_mut().copy_to_bytes(app_data_size)
        },
    };
    Ok(v)
}

fn decode_forward_fail<'de, D>(
    mut decoder: D,
    status: Status,
) -> Result<ForwardRequestFail, D::Error>
where
    D: Decoder<'de>,
{
    let is_routing_error = status.is_routing_error();
    let max_size = if is_routing_error { 9 } else { 8 };
    decoder.ensure_size(max_size)?;
    let res = ForwardRequestFail {
        connection_serial_number: decoder.decode_u16(),
        originator_vendor_id: decoder.decode_u16(),
        originator_serial_number: decoder.decode_u32(),
        remaining_path_size: if is_routing_error {
            Some(decoder.decode_u8())
        } else {
            None
        },
    };
    Ok(res)
}
