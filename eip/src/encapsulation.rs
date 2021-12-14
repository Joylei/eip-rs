// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::consts::{ENCAPSULATION_DATA_MAX_LEN, ENCAPSULATION_HEADER_LEN};
use bytes::Buf;
use rseip_core::{
    codec::{Decode, Encode, Encoder},
    hex::AsHex,
    Error,
};

/// UCMM: 504 bytes
/// max: 65535
#[derive(Debug, Default)]
pub struct EncapsulationPacket<T> {
    pub hdr: EncapsulationHeader,
    /// max length: 65511
    pub data: T,
}

/// header: 24 bytes
#[derive(Debug, Default)]
pub struct EncapsulationHeader {
    pub command: u16,
    /// Length, in bytes, of the data portion of the message
    pub length: u16,
    pub session_handle: u32,
    pub status: u32,
    pub sender_context: [u8; 8],
    /// shall be 0, receiver should ignore the command if not zero
    pub options: u32,
}

impl EncapsulationHeader {
    #[inline]
    pub fn ensure_command<E: Error>(&self, command_code: u16) -> Result<(), E> {
        if self.command != command_code {
            return Err(E::invalid_value(
                format_args!("command code {:#0x?}", self.command),
                command_code.as_hex(),
            ));
        }
        Ok(())
    }
}

impl<T: Encode> Encode for EncapsulationPacket<T> {
    #[inline]
    fn encode<A: Encoder>(
        mut self,
        buf: &mut bytes::BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        let data_len = self.data.bytes_count();
        debug_assert!(data_len <= ENCAPSULATION_DATA_MAX_LEN);

        self.hdr.length = data_len as u16;
        self.hdr.encode(buf, encoder)?;
        self.data.encode(buf, encoder)?;
        Ok(())
    }

    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut bytes::BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        let data_len = self.data.bytes_count();
        debug_assert!(data_len <= ENCAPSULATION_DATA_MAX_LEN);

        //encode hdr
        encoder.encode_u16(self.hdr.command, buf)?;
        encoder.encode_u16(data_len as u16, buf)?;
        encoder.encode_u32(self.hdr.session_handle, buf)?;
        encoder.encode_u32(self.hdr.status, buf)?;
        self.hdr.sender_context.encode_by_ref(buf, encoder)?;
        encoder.encode_u32(self.hdr.options, buf)?;

        //encode data
        self.data.encode_by_ref(buf, encoder)?;
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN + self.data.bytes_count()
    }
}

impl Encode for EncapsulationHeader {
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut bytes::BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        encoder.encode_u16(self.command, buf)?;
        encoder.encode_u16(self.length, buf)?;
        encoder.encode_u32(self.session_handle, buf)?;
        encoder.encode_u32(self.status, buf)?;
        self.sender_context.encode_by_ref(buf, encoder)?;
        encoder.encode_u32(self.options, buf)?;
        Ok(())
    }

    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut bytes::BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        encoder.encode_u16(self.command, buf)?;
        encoder.encode_u16(self.length, buf)?;
        encoder.encode_u32(self.session_handle, buf)?;
        encoder.encode_u32(self.status, buf)?;
        self.sender_context.encode_by_ref(buf, encoder)?;
        encoder.encode_u32(self.options, buf)?;
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN
    }
}

impl<'de> Decode<'de> for EncapsulationHeader {
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: rseip_core::codec::Decoder<'de>,
    {
        decoder.ensure_size(ENCAPSULATION_HEADER_LEN)?;
        let hdr = EncapsulationHeader {
            command: decoder.decode_u16(),
            length: decoder.decode_u16(),
            session_handle: decoder.decode_u32(),
            status: decoder.decode_u32(),
            sender_context: {
                let mut dst = [0; 8];
                decoder.buf_mut().copy_to_slice(&mut dst);
                dst
            },
            options: decoder.decode_u32(),
        };

        Ok(hdr)
    }
}
