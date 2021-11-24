use crate::cip;
use crate::{Error, Result};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, Bytes, BytesMut};
use rseip_cip::codec::Encodable;
use std::{convert::TryFrom, mem};

#[derive(Debug)]
pub enum TagValue<D = Bytes> {
    /// atomic data type: BOOL
    BOOL(bool),
    /// atomic data type: DWORD, 32-bit boolean array
    DWORD(u32),
    /// atomic data type: SINT, 8-bit integer
    SINT(i8),
    /// atomic data type: INT, 16-bit integer
    INT(i16),
    /// atomic data type: DINT, 32-bit integer
    DINT(i32),
    /// atomic data type: LINT, 64-bit integer
    LINT(i64),
    /// atomic data type: REAL, 32-bit float
    REAL(f32),
    UDT(D),
}

impl TryFrom<Bytes> for TagValue<Bytes> {
    type Error = Error;
    #[inline]
    fn try_from(src: Bytes) -> Result<Self> {
        //TODO: verify len
        assert!(src.len() >= 4);
        let tag_type = LittleEndian::read_u16(&src[0..2]);
        let val = match tag_type {
            0xC2 => TagValue::SINT(unsafe { mem::transmute(src[2]) }),
            0xC3 => TagValue::INT(LittleEndian::read_i16(&src[2..4])),
            0xC4 => {
                assert!(src.len() >= 6);
                TagValue::DINT(LittleEndian::read_i32(&src[2..6]))
            }
            0xCA => {
                assert!(src.len() >= 6);
                TagValue::REAL(LittleEndian::read_f32(&src[2..6]))
            }
            0xD3 => {
                assert!(src.len() >= 6);
                TagValue::DWORD(LittleEndian::read_u32(&src[2..6]))
            }
            0xC5 => {
                assert!(src.len() >= 10);
                TagValue::LINT(LittleEndian::read_i64(&src[2..10]))
            }
            0xC1 => TagValue::BOOL(src[4] == 255),
            _ => TagValue::UDT(src),
        };
        Ok(val)
    }
}

impl<D: Encodable> Encodable for TagValue<D> {
    #[inline]
    fn encode(self, dst: &mut BytesMut) -> cip::Result<()> {
        match self {
            Self::BOOL(v) => {
                dst.put_u16_le(0xC1);
                dst.put_slice(&[1, 0]);
                dst.put_u8(if v { 255 } else { 0 });
                dst.put_u8(0);
            }
            Self::SINT(v) => {
                dst.put_u16_le(0xC2);
                dst.put_slice(&[1, 0]);
                dst.put_i8(v);
                dst.put_u8(0);
            }
            Self::INT(v) => {
                dst.put_u16_le(0xC3);
                dst.put_slice(&[1, 0]);
                dst.put_i16_le(v);
            }
            Self::DINT(v) => {
                dst.put_u16_le(0xC4);
                dst.put_slice(&[1, 0]);
                dst.put_i32_le(v);
            }
            Self::REAL(v) => {
                dst.put_u16_le(0xCA);
                dst.put_slice(&[1, 0]);
                dst.put_f32_le(v);
            }
            Self::DWORD(v) => {
                dst.put_u16_le(0xD3);
                dst.put_slice(&[1, 0]);
                dst.put_u32_le(v);
            }
            Self::LINT(v) => {
                dst.put_u16_le(0xC5);
                dst.put_slice(&[1, 0]);
                dst.put_i64_le(v);
            }
            Self::UDT(data) => {
                data.encode(dst)?;
            }
        };
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        match self {
            Self::BOOL(_) => 6,
            Self::SINT(_) => 6,
            Self::INT(_) => 6,
            Self::DINT(_) => 8,
            Self::REAL(_) => 8,
            Self::DWORD(_) => 8,
            Self::LINT(_) => 12,
            Self::UDT(v) => v.bytes_count(),
        }
    }
}
