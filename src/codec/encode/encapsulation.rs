use crate::{
    codec::Encodable,
    consts::{ENCAPSULATION_DATA_MAX_LEN, ENCAPSULATION_HEADER_LEN},
    frame::{EncapsulationHeader, EncapsulationPacket},
    Result,
};
use bytes::{BufMut, BytesMut};

impl<D: Encodable> Encodable for EncapsulationPacket<D> {
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

impl Encodable for EncapsulationHeader {
    #[inline(always)]
    fn encode(self, dst: &mut bytes::BytesMut) -> Result<()> {
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
