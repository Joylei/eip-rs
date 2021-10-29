// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2020-2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{
    codec::Encodable,
    frame::cip::{EPath, PortSegment, Segment},
    Result,
};
use bytes::{BufMut, BytesMut};

impl Encodable for PortSegment {
    #[inline]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        const EXTENDED_LINKED_ADDRESS_SIZE: u16 = 1 << 4; // 0x10

        let link_addr_len = self.link.len();
        assert!(link_addr_len < u8::MAX as usize);

        let start_pos = dst.len();
        let mut segment_byte = if self.port > 14 { 0x0F } else { self.port };
        if link_addr_len > 1 {
            segment_byte |= EXTENDED_LINKED_ADDRESS_SIZE;
        }
        dst.put_u8(segment_byte as u8);
        if link_addr_len > 1 {
            dst.put_u8(link_addr_len as u8);
        }
        if self.port > 14 {
            dst.put_u16(self.port);
        }

        dst.put_slice(&self.link);
        let end_pos = dst.len();
        if (end_pos - start_pos) % 2 != 0 {
            dst.put_u8(0);
        }
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        let link_addr_len = self.link.len();
        let mut count = 1;
        if link_addr_len > 1 {
            count += 1;
        }
        if self.port > 14 {
            count += 2;
        }
        count += link_addr_len;
        count + count % 2
    }
}

impl Encodable for Segment {
    fn bytes_count(&self) -> usize {
        match self {
            Segment::Class(v) | Segment::Instance(v) | Segment::Attribute(v) => {
                if *v <= u8::MAX as u16 {
                    2
                } else {
                    4
                }
            }
            Segment::Element(elem) => match elem {
                v if *v <= (u8::MAX as u32) => 2,
                v if *v <= (u16::MAX as u32) => 4,
                _ => 6,
            },
            Segment::Port(port) => port.bytes_count(),
            Segment::Symbol(symbol) => {
                let char_count = symbol.as_bytes().len();
                2 + char_count + char_count % 2
            }
        }
    }

    #[inline]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        match self {
            Segment::Class(v) => {
                if v <= u8::MAX as u16 {
                    dst.put_u8(0x20);
                    dst.put_u8(v as u8);
                } else {
                    dst.put_u8(0x21);
                    dst.put_u8(0);
                    dst.put_u16_le(v);
                }
            }
            Segment::Instance(v) => {
                if v <= u8::MAX as u16 {
                    dst.put_u8(0x24);
                    dst.put_u8(v as u8);
                } else {
                    dst.put_u8(0x25);
                    dst.put_u8(0);
                    dst.put_u16_le(v);
                }
            }
            Segment::Attribute(v) => {
                if v <= u8::MAX as u16 {
                    dst.put_u8(0x30);
                    dst.put_u8(v as u8);
                } else {
                    dst.put_u8(0x31);
                    dst.put_u8(0);
                    dst.put_u16_le(v);
                }
            }
            Segment::Element(elem) => match elem {
                v if v <= (u8::MAX as u32) => {
                    dst.put_u8(0x28);
                    dst.put_u8(v as u8);
                }
                v if v <= (u16::MAX as u32) => {
                    dst.put_u8(0x29);
                    dst.put_u8(0);
                    dst.put_u16_le(v as u16);
                }
                v => {
                    dst.put_u8(0x2A);
                    dst.put_u8(0);
                    dst.put_u32_le(v);
                }
            },
            Segment::Port(port) => port.encode(dst)?,
            Segment::Symbol(symbol) => {
                let char_count = symbol.as_bytes().len();
                assert!(char_count <= u8::MAX as usize);
                dst.put_u8(0x91);
                dst.put_u8(char_count as u8);
                dst.put_slice(symbol.as_bytes());
                if char_count % 2 != 0 {
                    dst.put_u8(0);
                }
            }
        }
        Ok(())
    }
}

impl Encodable for EPath {
    #[inline(always)]
    fn encode(self: EPath, dst: &mut BytesMut) -> Result<()> {
        for item in self.into_vec() {
            item.encode(dst)?;
        }
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.iter().map(|v| v.bytes_count()).sum()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::frame::cip::epath::EPATH_CONNECTION_MANAGER;

    #[test]
    fn test_epath_symbol() {
        let epath = EPath::from(vec![Segment::Symbol("TotalCount".to_owned())]);

        assert_eq!(epath.bytes_count(), 12);

        let mut buf = BytesMut::new();
        epath.encode(&mut buf).unwrap();

        assert_eq!(
            &buf[..],
            &[0x91, 0x0A, 0x54, 0x6F, 0x74, 0x61, 0x6C, 0x43, 0x6F, 0x75, 0x6E, 0x74]
        );
    }

    #[test]
    fn test_epath_symbol_odd() {
        let epath = EPath::from(vec![Segment::Symbol("TotalCountt".to_owned())]);

        assert_eq!(epath.bytes_count(), 14);

        let mut buf = BytesMut::new();
        epath.encode(&mut buf).unwrap();

        assert_eq!(
            &buf[..],
            &[0x91, 0x0B, 0x54, 0x6F, 0x74, 0x61, 0x6C, 0x43, 0x6F, 0x75, 0x6E, 0x74, 0x74, 0x00]
        );
    }

    #[test]
    fn test_epath_unconnected_send() {
        let epath = EPath::from(vec![Segment::Class(0x06), Segment::Instance(0x1)]);

        assert_eq!(epath.bytes_count(), 4);

        let mut buf = BytesMut::new();
        epath.encode(&mut buf).unwrap();

        assert_eq!(&buf[..], EPATH_CONNECTION_MANAGER);
    }

    #[test]
    fn test_epath_router_path() {
        let epath = EPath::from(vec![Segment::Port(PortSegment::default())]);

        assert_eq!(epath.bytes_count(), 2);

        let mut buf = BytesMut::new();
        epath.encode(&mut buf).unwrap();

        assert_eq!(&buf[..], &[01, 00]);
    }
}
