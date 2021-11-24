// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::codec::Encoding;
use crate::{
    consts::{ENCAPSULATION_DATA_MAX_LEN, ENCAPSULATION_HEADER_LEN},
    EncapsulationHeader, EncapsulationPacket, Result,
};
use bytes::{BufMut, BytesMut};

impl<D: Encoding> Encoding for EncapsulationPacket<D> {
    #[inline(always)]
    fn encode(mut self, dst: &mut BytesMut) -> Result<()> {
        let data_len = self.data.bytes_count();
        assert!(data_len <= ENCAPSULATION_DATA_MAX_LEN);

        self.hdr.length = data_len as u16;
        dst.reserve(ENCAPSULATION_HEADER_LEN);
        self.hdr.encode(dst)?;
        self.data.encode(dst)?;
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN + self.data.bytes_count()
    }
}

impl Encoding for EncapsulationHeader {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        dst.put_u16_le(self.command);
        dst.put_u16_le(self.length);
        dst.put_u32_le(self.session_handle);
        dst.put_u32_le(self.status);
        dst.put_slice(&self.sender_context);
        dst.put_u32_le(self.options);
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN
    }
}
