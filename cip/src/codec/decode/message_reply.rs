// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::*;
use crate::{
    connection::*,
    error::{cip_error, cip_error_status},
    Status,
};
use bytes::Buf;
use rseip_core::{
    cip::CommonPacketIter,
    codec::{Decode, Decoder},
    Either, Error,
};

#[inline]
pub fn decode_service_and_status<'de, D>(mut decoder: D) -> Result<(u8, Status), D::Error>
where
    D: Decoder<'de>,
{
    decoder.ensure_size(4)?;
    let reply_service = decoder.decode_u8(); // buf[0]
    decoder.buf_mut().advance(1); // buf[1]
    let general_status = decoder.decode_u8(); //buf[2]
    let extended_status_size = decoder.decode_u8(); // buf[3]
    decoder.ensure_size((extended_status_size * 2) as usize)?;
    let extended_status = match extended_status_size {
        0 => None,
        1 => Some(decoder.decode_u16()),
        v => return Err(Error::invalid_value("one of 0,1", v)),
    };
    let status = Status {
        general: general_status,
        extended: extended_status,
    };

    Ok((reply_service, status))
}

impl<'de, R> Decode<'de> for MessageReply<R>
where
    R: Decode<'de>,
{
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let (reply_service, status) = decode_service_and_status(&mut decoder)?;
        if status.is_err() {
            return Err(cip_error_status(status));
        }
        let data = decoder.decode_any()?;
        Ok(MessageReply::new(reply_service, status, data))
    }
}

/// decode message reply from common packet for connected send
///
/// ```rust, ignore
/// let data:(u16, MessageReply<T>) = decode_connected_send(cpf)?;
/// ````
#[inline]
pub fn decode_connected_send<'de, D, R>(
    mut cpf: CommonPacketIter<'de, D>,
) -> Result<(u16, R), D::Error>
where
    D: Decoder<'de>,
    R: Decode<'de> + 'static,
{
    if cpf.len() != 2 {
        return Err(cip_error("common packet - expect 2 items"));
    }
    // should be connected address
    ensure_connected_address(&mut cpf)?;
    // should be connected data item
    if let Some(res) = cpf.next_typed() {
        let data_item: CommonPacketItem<(u16, _)> = res?;
        data_item.ensure_type_code::<D::Error>(0xB1)?;
        return Ok(data_item.data);
    }
    Err(cip_error("common packet - expect connected data item"))
}

/// decode message reply from common packet for unconnected send
///
/// ```rust, ignore
/// let data:MessageReply<T>= decode_unconnected_send(cpf)?;
/// ```
#[inline]
pub fn decode_unconnected_send<'de, D, R>(mut cpf: CommonPacketIter<'de, D>) -> Result<R, D::Error>
where
    D: Decoder<'de>,
    R: Decode<'de> + 'static,
{
    if cpf.len() != 2 {
        return Err(cip_error("common packet - expected 2 items"));
    }
    // should be null address
    ensure_null_address(&mut cpf)?;

    // should be unconnected data item
    if let Some(res) = cpf.next_typed() {
        let data_item: CommonPacketItem<_> = res?;
        data_item.ensure_type_code::<D::Error>(0xB2)?;
        return Ok(data_item.data);
    }
    Err(cip_error("common packet - expect connected data item"))
}

impl<'de> Decode<'de> for ForwardOpenReply {
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let (reply_service, status) = decode_service_and_status(&mut decoder)?;
        let data = if status.is_err() {
            let v = decode_forward_fail(decoder, status)?;
            Either::Right(v)
        } else {
            let v = decode_forward_open_success(decoder)?;
            Either::Left(v)
        };
        Ok(Self(MessageReply::new(reply_service, status, data)))
    }
}

impl<'de> Decode<'de> for ForwardCloseReply {
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let (reply_service, status) = decode_service_and_status(&mut decoder)?;
        let data = if status.is_err() {
            let v = decode_forward_fail(decoder, status)?;
            Either::Right(v)
        } else {
            let v = decode_forward_close_success(decoder)?;
            Either::Left(v)
        };
        Ok(Self(MessageReply::new(reply_service, status, data)))
    }
}

#[inline]
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

#[inline]
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

#[inline]
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

#[inline]
pub fn ensure_null_address<'de, D>(cpf: &mut CommonPacketIter<'de, D>) -> Result<(), D::Error>
where
    D: Decoder<'de>,
{
    if let Some(res) = cpf.next_item() {
        let item = res?;
        if item.type_code == 0 {
            return Ok(());
        }
    }
    Err(cip_error("common packet - expect null address"))
}

#[inline]
pub fn ensure_connected_address<'de, D>(cpf: &mut CommonPacketIter<'de, D>) -> Result<(), D::Error>
where
    D: Decoder<'de>,
{
    if let Some(res) = cpf.next_item() {
        let item = res?;
        if item.type_code == 0xA1 {
            return Ok(());
        }
    }
    Err(cip_error("common packet - expect null address"))
}
