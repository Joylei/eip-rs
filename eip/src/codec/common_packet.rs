// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{codec::Encoding, CommonPacket, CommonPacketItem, Result};
use bytes::{BufMut, BytesMut};

impl Encoding for CommonPacket {
    #[inline]
    fn encode(self: CommonPacket, dst: &mut BytesMut) -> Result<()> {
        debug_assert!(self.len() > 0 && self.len() <= 4);
        dst.put_u16_le(self.len() as u16);
        for item in self.into_iter() {
            item.encode(dst)?;
        }
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        let count: usize = self.iter().map(|v| v.bytes_count()).sum();
        count + 2
    }
}

impl Encoding for CommonPacketItem {
    #[inline]
    fn encode(self: CommonPacketItem, dst: &mut BytesMut) -> Result<()> {
        let bytes_count = self.bytes_count();
        dst.reserve(bytes_count);
        dst.put_u16_le(self.type_code);
        debug_assert!(self.data.len() <= u16::MAX as usize);
        dst.put_u16_le(self.data.len() as u16);
        dst.put_slice(&self.data);
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        4 + self.data.len()
    }
}

#[cfg(test)]
mod test {
    use bytes::Bytes;

    use super::*;

    #[test]
    fn test_common_packet_item() {
        let item = CommonPacketItem {
            type_code: 0x00,
            data: Bytes::from_static(&[0, 0]),
        };
        assert_eq!(item.bytes_count(), 6);
        let buf = item.try_into_bytes().unwrap();
        assert_eq!(&buf[..], &[0x00, 0x00, 0x02, 0x00, 0x00, 0x00,]);
    }

    #[test]
    fn test_common_packet() {
        let null_addr = CommonPacketItem {
            type_code: 0x00,
            data: Bytes::from_static(&[0, 0]),
        };
        let data_item = CommonPacketItem {
            type_code: 0xB2,
            data: Bytes::from_static(&[
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
            ]),
        };
        let cpf = CommonPacket::from(vec![null_addr, data_item]);
        assert_eq!(cpf.bytes_count(), 2 + 4 + 2 + 4 + 9);
        let buf = cpf.try_into_bytes().unwrap();
        assert_eq!(
            &buf[..],
            &[
                0x02, 0x00, // item count
                0x00, 0x00, 0x02, 0x00, 0x00, 0x00, // addr item
                0xB2, 0x00, 0x09, 0x00, 1, 2, 3, 4, 5, 6, 7, 8, 9, //data item
            ]
        );
    }
}
