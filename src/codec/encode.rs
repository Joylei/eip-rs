use bytes::{BufMut, BytesMut};
use tokio_util::codec::Encoder;

use crate::consts::ENCAPSULATION_HEADER_LEN;
use crate::error::Error;
use crate::frame::common_packet::{CommonPacketFormat, CommonPacketItem};
use crate::frame::encapsulation::{EncapsulationHeader, EncapsulationPacket};
use crate::frame::Request;
use crate::objects::socket::SocketAddr;

use super::{ClientCodec, EncodedBytesCount};

impl Encoder<Request> for ClientCodec {
    type Error = Error;
    fn encode(&mut self, item: Request, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let pkt = EncapsulationPacket::from(item);
        self.encode(pkt, dst)
    }
}

impl Encoder<EncapsulationPacket> for ClientCodec {
    type Error = Error;
    #[inline(always)]
    fn encode(
        &mut self,
        mut item: EncapsulationPacket,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        let data_len = item.data.as_ref().map(|v| v.len()).unwrap_or_default();
        item.hdr.length = data_len as u16;
        dst.reserve(ENCAPSULATION_HEADER_LEN + data_len);
        self.encode(item.hdr, dst)?;
        if let Some(data) = item.data {
            dst.put_slice(&*data);
        }
        Ok(())
    }
}

impl Encoder<EncapsulationHeader> for ClientCodec {
    type Error = Error;
    #[inline(always)]
    fn encode(
        &mut self,
        item: EncapsulationHeader,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        dst.put_u16_le(item.command);
        dst.put_u16_le(item.length);
        dst.put_u32_le(item.session_handler);
        dst.put_u32_le(item.status);
        dst.put_slice(&item.sender_context);
        dst.put_u32_le(item.options);
        Ok(())
    }
}

impl Encoder<CommonPacketFormat> for ClientCodec {
    type Error = Error;
    #[inline(always)]
    fn encode(&mut self, item: CommonPacketFormat, dst: &mut BytesMut) -> Result<(), Self::Error> {
        debug_assert!(item.len() > 0 && item.len() <= 4);
        dst.put_u16_le(item.len() as u16);
        for item in item.into_vec() {
            self.encode(item, dst)?;
        }
        Ok(())
    }
}

impl Encoder<CommonPacketItem> for ClientCodec {
    type Error = Error;
    #[inline(always)]
    fn encode(&mut self, item: CommonPacketItem, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes_count = item.bytes_count();
        dst.reserve(bytes_count);
        dst.put_u16_le(item.type_code);
        if let Some(data) = item.data {
            debug_assert!(data.len() <= u16::MAX as usize);
            dst.put_u16_le(data.len() as u16);
            dst.put_slice(&data);
        } else {
            dst.put_u16_le(0);
        }
        Ok(())
    }
}

impl Encoder<SocketAddr> for ClientCodec {
    type Error = Error;
    #[inline(always)]
    fn encode(&mut self, addr: SocketAddr, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_i16(addr.sin_family);
        dst.put_u16(addr.sin_port);
        dst.put_u32(addr.sin_addr);
        dst.put_slice(&addr.sin_zero);
        Ok(())
    }
}
