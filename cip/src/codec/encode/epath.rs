// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::epath::*;
use bytes::{BufMut, BytesMut};
use rseip_core::codec::{Encode, Encoder};

impl Encode for PortSegment {
    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error> {
        const EXTENDED_LINKED_ADDRESS_SIZE: u16 = 1 << 4; // 0x10

        let link_addr_len = self.link.len();
        debug_assert!(link_addr_len < u8::MAX as usize);

        let start_pos = buf.len();
        let mut segment_byte = if self.port > 14 { 0x0F } else { self.port };
        if link_addr_len > 1 {
            segment_byte |= EXTENDED_LINKED_ADDRESS_SIZE;
        }
        buf.put_u8(segment_byte as u8);
        if link_addr_len > 1 {
            buf.put_u8(link_addr_len as u8);
        }
        if self.port > 14 {
            buf.put_u16(self.port);
        }

        buf.put_slice(&self.link);
        let end_pos = buf.len();
        if (end_pos - start_pos) % 2 != 0 {
            buf.put_u8(0);
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

impl Segment {
    #[inline]
    fn encode_class<A: Encoder>(
        v: u16,
        buf: &mut BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error> {
        if v <= u8::MAX as u16 {
            buf.put_u8(0x20);
            buf.put_u8(v as u8);
        } else {
            buf.put_u8(0x21);
            buf.put_u8(0);
            buf.put_u16_le(v);
        }
        Ok(())
    }

    #[inline]
    fn encode_instance<A: Encoder>(
        v: u16,
        buf: &mut BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error> {
        if v <= u8::MAX as u16 {
            buf.put_u8(0x24);
            buf.put_u8(v as u8);
        } else {
            buf.put_u8(0x25);
            buf.put_u8(0);
            buf.put_u16_le(v);
        }
        Ok(())
    }

    #[inline]
    fn encode_attribute<A: Encoder>(
        v: u16,
        buf: &mut BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error> {
        if v <= u8::MAX as u16 {
            buf.put_u8(0x30);
            buf.put_u8(v as u8);
        } else {
            buf.put_u8(0x31);
            buf.put_u8(0);
            buf.put_u16_le(v);
        }
        Ok(())
    }

    #[inline]
    fn encode_element<A: Encoder>(
        elem: u32,
        buf: &mut BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error> {
        match elem {
            v if v <= (u8::MAX as u32) => {
                buf.put_u8(0x28);
                buf.put_u8(v as u8);
            }
            v if v <= (u16::MAX as u32) => {
                buf.put_u8(0x29);
                buf.put_u8(0);
                buf.put_u16_le(v as u16);
            }
            v => {
                buf.put_u8(0x2A);
                buf.put_u8(0);
                buf.put_u32_le(v);
            }
        }
        Ok(())
    }

    #[inline]
    fn encode_symbol<A: Encoder>(
        symbol: &[u8],
        buf: &mut BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error> {
        let char_count = symbol.len();
        assert!(char_count <= u8::MAX as usize);
        buf.put_u8(0x91);
        buf.put_u8(char_count as u8);
        buf.put_slice(symbol);
        if char_count % 2 != 0 {
            buf.put_u8(0);
        }
        Ok(())
    }
}

impl Encode for Segment {
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error> {
        match self {
            Segment::Class(v) => Self::encode_class(v, buf, encoder),
            Segment::Instance(v) => Self::encode_instance(v, buf, encoder),
            Segment::Attribute(v) => Self::encode_attribute(v, buf, encoder),
            Segment::Element(v) => Self::encode_element(v, buf, encoder),
            Segment::Port(port) => port.encode_by_ref(buf, encoder),
            Segment::Symbol(symbol) => Self::encode_symbol(symbol.as_bytes(), buf, encoder),
        }
    }

    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        match self {
            Segment::Class(v) => Self::encode_class(*v, buf, encoder),
            Segment::Instance(v) => Self::encode_instance(*v, buf, encoder),
            Segment::Attribute(v) => Self::encode_attribute(*v, buf, encoder),
            Segment::Element(v) => Self::encode_element(*v, buf, encoder),
            Segment::Port(port) => port.encode_by_ref(buf, encoder),
            Segment::Symbol(symbol) => Self::encode_symbol(symbol.as_bytes(), buf, encoder),
        }
    }

    #[inline]
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
}

impl Encode for EPath {
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error> {
        for item in self {
            item.encode(buf, encoder)?;
        }
        Ok(())
    }
    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        for item in self.iter() {
            item.encode_by_ref(buf, encoder)?;
        }
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        self.iter().map(|v| v.bytes_count()).sum()
    }
}

impl From<PortSegment> for EPath {
    fn from(port: PortSegment) -> Self {
        Self::from(vec![Segment::Port(port)])
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::epath::EPATH_CONNECTION_MANAGER;
    use rseip_core::tests::EncodeExt;

    #[test]
    fn test_epath_symbol() {
        let epath = EPath::from_symbol("TotalCount");

        assert_eq!(epath.bytes_count(), 12);

        let buf = epath.try_into_bytes().unwrap();

        assert_eq!(
            &buf[..],
            &[0x91, 0x0A, 0x54, 0x6F, 0x74, 0x61, 0x6C, 0x43, 0x6F, 0x75, 0x6E, 0x74]
        );
    }

    #[test]
    fn test_epath_symbol_odd() {
        let epath = EPath::from_symbol("TotalCountt");
        assert_eq!(epath.bytes_count(), 14);

        let buf = epath.try_into_bytes().unwrap();

        assert_eq!(
            &buf[..],
            &[0x91, 0x0B, 0x54, 0x6F, 0x74, 0x61, 0x6C, 0x43, 0x6F, 0x75, 0x6E, 0x74, 0x74, 0x00]
        );
    }

    #[test]
    fn test_epath_unconnected_send() {
        let epath = EPath::from(vec![Segment::Class(0x06), Segment::Instance(0x1)]);

        assert_eq!(epath.bytes_count(), 4);

        let buf = epath.try_into_bytes().unwrap();

        assert_eq!(&buf[..], EPATH_CONNECTION_MANAGER);
    }

    #[test]
    fn test_epath_router_path() {
        let epath = EPath::from(vec![Segment::Port(PortSegment::default())]);

        assert_eq!(epath.bytes_count(), 2);

        let buf = epath.try_into_bytes().unwrap();

        assert_eq!(&buf[..], &[01, 00]);
    }
}
