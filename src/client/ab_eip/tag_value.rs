use crate::{
    cip::{self, codec::Encodable},
    Error, Result,
};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{convert::TryFrom, mem};

#[derive(Debug, Clone, Copy)]
pub enum TagType {
    /// atomic data type: BOOL
    BOOL,
    /// atomic data type: DWORD, 32-bit boolean array
    DWORD,
    /// atomic data type: SINT, 8-bit integer
    SINT,
    /// atomic data type: INT, 16-bit integer
    INT,
    /// atomic data type: DINT, 32-bit integer
    DINT,
    /// atomic data type: LINT, 64-bit integer
    LINT,
    /// atomic data type: REAL, 32-bit float
    REAL,
    /// structured tag
    Structure(u16),
}

impl TagType {
    /// two bytes type code
    #[inline]
    pub fn type_code(&self) -> u16 {
        match self {
            Self::BOOL => 0xC1,
            Self::DWORD => 0xD3,
            Self::SINT => 0xC2,
            Self::INT => 0xC3,
            Self::DINT => 0xC4,
            Self::LINT => 0xC5,
            Self::REAL => 0xCA,
            Self::Structure { .. } => 0x02A0,
        }
    }

    /// is it a structure
    pub fn is_structure(&self) -> bool {
        match self {
            Self::Structure(_) => true,
            _ => false,
        }
    }

    /// is it a atomic type
    pub fn is_atomic(&self) -> bool {
        !self.is_structure()
    }

    /// get structure handle if it's a structure
    pub fn structure_handle(&self) -> Option<u16> {
        match self {
            Self::Structure(v) => Some(*v),
            _ => None,
        }
    }
}

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
    /// structured tag
    Structure {
        /// type handle
        handle: u16,
        /// tag data
        data: D,
    },
}

impl<D> TagValue<D> {
    /// get tag type
    #[inline]
    pub fn tag_type(&self) -> TagType {
        match self {
            Self::BOOL(_) => TagType::BOOL,
            Self::DWORD(_) => TagType::DWORD,
            Self::SINT(_) => TagType::SINT,
            Self::INT(_) => TagType::INT,
            Self::DINT(_) => TagType::DINT,
            Self::LINT(_) => TagType::LINT,
            Self::REAL(_) => TagType::REAL,
            Self::Structure { handle, .. } => TagType::Structure(*handle),
        }
    }
}

impl TryFrom<Bytes> for TagValue<Bytes> {
    type Error = Error;
    #[inline]
    fn try_from(mut src: Bytes) -> Result<Self> {
        //TODO: verify len
        assert!(src.len() >= 4);
        let type_code = LittleEndian::read_u16(&src[0..2]);
        let val = match type_code {
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
            0x02A0 => {
                assert!(src.len() > 4);
                TagValue::Structure {
                    handle: src.get_u16_le(),
                    data: src,
                }
            }
            _ => unreachable!("unexpected type code: {}", type_code),
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
            Self::Structure { handle, data } => {
                dst.put_slice(&[0xA0, 0x02]);
                dst.put_u16_le(handle);
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
            Self::Structure { data, .. } => 4 + data.bytes_count(),
        }
    }
}
