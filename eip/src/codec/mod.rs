// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod command;

use crate::{
    consts::*,
    error::{eip_error, eip_error_code},
    EncapsulationHeader, EncapsulationPacket,
};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, Bytes, BytesMut};
use core::marker::PhantomData;
use rseip_core::{
    codec::{self, Decode, Encode, LittleEndianDecoder},
    Error,
};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, PartialEq)]
pub struct ClientCodec<E> {
    _marker: PhantomData<E>,
}

impl<E> ClientCodec<E> {
    pub(crate) fn new() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<E: Error> codec::Encoder for ClientCodec<E> {
    type Error = E;

    #[inline(always)]
    fn encode_bool(&mut self, item: bool, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put_u8(if item { 255 } else { 0 });
        Ok(())
    }

    #[inline(always)]
    fn encode_i8(&mut self, item: i8, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put_i8(item);
        Ok(())
    }

    #[inline(always)]
    fn encode_u8(&mut self, item: u8, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put_u8(item);
        Ok(())
    }

    #[inline(always)]
    fn encode_i16(&mut self, item: i16, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put_i16_le(item);
        Ok(())
    }

    #[inline(always)]
    fn encode_u16(&mut self, item: u16, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put_u16_le(item);
        Ok(())
    }

    #[inline(always)]
    fn encode_i32(&mut self, item: i32, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put_i32_le(item);
        Ok(())
    }

    #[inline(always)]
    fn encode_u32(&mut self, item: u32, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put_u32_le(item);
        Ok(())
    }

    #[inline(always)]
    fn encode_i64(&mut self, item: i64, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put_i64_le(item);
        Ok(())
    }

    #[inline(always)]
    fn encode_u64(&mut self, item: u64, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put_u64_le(item);
        Ok(())
    }

    #[inline(always)]
    fn encode_f32(&mut self, item: f32, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put_f32_le(item);
        Ok(())
    }

    #[inline(always)]
    fn encode_f64(&mut self, item: f64, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put_f64_le(item);
        Ok(())
    }

    #[inline(always)]
    fn encode_i128(&mut self, item: i128, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put_i128_le(item);
        Ok(())
    }

    #[inline(always)]
    fn encode_u128(&mut self, item: u128, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put_u128_le(item);
        Ok(())
    }
}

impl<I, E> Encoder<EncapsulationPacket<I>> for ClientCodec<E>
where
    I: codec::Encode + Sized,
    E: Error,
{
    type Error = E;
    #[inline]
    fn encode(
        &mut self,
        item: EncapsulationPacket<I>,
        buf: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        item.encode(buf, self)
    }
}

impl<E: Error> Decoder for ClientCodec<E> {
    type Error = E;
    type Item = EncapsulationPacket<Bytes>;
    #[inline]
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < ENCAPSULATION_HEADER_LEN {
            return Ok(None);
        }
        let data_len = LittleEndian::read_u16(&src[2..4]) as usize;
        //verify data length
        if ENCAPSULATION_HEADER_LEN + data_len > u16::MAX as usize {
            return Err(E::invalid_length(
                ENCAPSULATION_HEADER_LEN + data_len,
                "below u16::MAX",
            ));
        }
        if src.len() < ENCAPSULATION_HEADER_LEN + data_len {
            return Ok(None);
        }
        if src.len() > ENCAPSULATION_HEADER_LEN + data_len {
            // should no remaining buffer
            return Err(E::invalid_length(
                src.len(),
                ENCAPSULATION_HEADER_LEN + data_len,
            ));
        }
        let header_bytes = src.split_to(ENCAPSULATION_HEADER_LEN).freeze();
        let decoder = LittleEndianDecoder::<E>::new(header_bytes);
        let hdr = EncapsulationHeader::decode(decoder)?;
        match hdr.status {
            0 => {}
            v if v > u16::MAX as u32 => {
                return Err(eip_error(format_args!("invalid status code {:#04x?}", v)));
            }
            v => return Err(eip_error_code(v as u16)),
        }
        let data = src.split_to(data_len).freeze();
        Ok(Some(EncapsulationPacket { hdr, data }))
    }
}
