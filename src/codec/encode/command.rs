// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2020-2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::LazyEncode;
use crate::{
    codec::Encodable,
    consts::ENCAPSULATION_HEADER_LEN,
    frame::{command::*, EncapsulationPacket},
    Result,
};
use bytes::{BufMut, BytesMut};

impl<D: Encodable> Encodable for Nop<D> {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        let mut pkt = EncapsulationPacket {
            hdr: Default::default(),
            data: self.data,
        };
        pkt.hdr.command = Self::command_code();
        pkt.encode(dst)
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN + self.data.bytes_count()
    }
}

impl Encodable for ListIdentity {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        let mut pkt = EncapsulationPacket::default();
        pkt.hdr.command = Self::command_code();
        pkt.data = ();
        pkt.encode(dst)
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN
    }
}

impl Encodable for ListInterfaces {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        let mut pkt = EncapsulationPacket::default();
        pkt.hdr.command = Self::command_code();
        pkt.data = ();
        pkt.encode(dst)
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN
    }
}

impl Encodable for ListServices {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        let mut pkt = EncapsulationPacket::default();
        pkt.hdr.command = Self::command_code();
        pkt.data = ();
        pkt.encode(dst)
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN
    }
}

impl Encodable for RegisterSession {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        let mut pkt: EncapsulationPacket<&[u8]> = EncapsulationPacket::default();
        pkt.hdr.command = Self::command_code();
        pkt.data = &[0x01, 0x00, 0x00, 0x00];
        pkt.encode(dst)
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN + 4
    }
}

impl Encodable for UnRegisterSession {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        let mut pkt = EncapsulationPacket::default();
        pkt.hdr.command = Self::command_code();
        pkt.hdr.session_handle = self.session_handle;
        pkt.data = ();
        pkt.encode(dst)
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN
    }
}

impl<D: Encodable> Encodable for SendRRData<D> {
    #[inline]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        let timeout = self.timeout;
        let data_item = self.data;
        let data_item_len = data_item.bytes_count();
        let data = LazyEncode {
            f: move |buf: &mut BytesMut| {
                buf.put_u32_le(0); // interface handle, shall be 0 for CIP
                buf.put_u16_le(timeout); // timeout, 0 for SendUnitData
                buf.put_u16_le(2); //  cpf item count
                buf.put_slice(&[0, 0, 0, 0]); // null address
                buf.put_u16_le(0xB2); // unconnected data item
                buf.put_u16_le(data_item_len as u16);
                data_item.encode(buf)?;
                Ok(())
            },
            bytes_count: 16 + data_item_len,
        };
        let mut pkt = EncapsulationPacket {
            hdr: Default::default(),
            data,
        };
        pkt.hdr.command = Self::command_code();
        pkt.hdr.session_handle = self.session_handle;
        pkt.encode(dst)
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN + 16 + self.data.bytes_count()
    }
}

impl<D: Encodable> Encodable for SendUnitData<D> {
    #[inline]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        let Self {
            session_handle,
            connection_id,
            sequence_number,
            data: data_item,
        } = self;
        let data_item_len = data_item.bytes_count();
        let data = LazyEncode {
            f: move |buf: &mut BytesMut| {
                buf.put_u32_le(0); // interface handle, shall be 0 for CIP
                buf.put_u16_le(0); // timeout, 0 for SendUnitData
                buf.put_u16_le(2); //  cpf item count

                buf.put_u16_le(0xA1); // connected address item
                buf.put_u16_le(4); // data len
                buf.put_u32_le(connection_id);

                buf.put_u16_le(0xB1); // connected data item
                buf.put_u16_le(data_item_len as u16 + 2); // data item len
                                                          //buf.put_u32_le(sequence_number.unwrap()); // sequence number

                buf.put_u16_le(sequence_number);
                data_item.encode(buf)?; // data request
                Ok(())
            },
            bytes_count: 22 + data_item_len,
        };
        let mut pkt = EncapsulationPacket {
            hdr: Default::default(),
            data,
        };
        pkt.hdr.command = Self::command_code();
        pkt.hdr.session_handle = session_handle;
        pkt.encode(dst)
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN + 22 + self.data.bytes_count()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_list_identity_request() {
        let buf = ListIdentity.try_into_bytes().unwrap();
        assert_eq!(
            &buf[..],
            &[
                0x63, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            ]
        )
    }
}
