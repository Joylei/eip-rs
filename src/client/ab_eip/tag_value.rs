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
    Bool,
    /// atomic data type: DWORD, 32-bit boolean array
    Dword,
    /// atomic data type: SINT, 8-bit integer
    Sint,
    /// atomic data type: INT, 16-bit integer
    Int,
    /// atomic data type: DINT, 32-bit integer
    Dint,
    /// atomic data type: LINT, 64-bit integer
    Lint,
    /// atomic data type: REAL, 32-bit float
    Real,
    /// structured tag
    Structure(u16),
}

impl TagType {
    /// two bytes type code
    #[inline]
    pub fn type_code(&self) -> u16 {
        match self {
            Self::Bool => 0xC1,
            Self::Dword => 0xD3,
            Self::Sint => 0xC2,
            Self::Int => 0xC3,
            Self::Dint => 0xC4,
            Self::Lint => 0xC5,
            Self::Real => 0xCA,
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
    Bool(bool),
    /// atomic data type: DWORD, 32-bit boolean array
    Dword(u32),
    /// atomic data type: SINT, 8-bit integer
    Sint(i8),
    /// atomic data type: INT, 16-bit integer
    Int(i16),
    /// atomic data type: DINT, 32-bit integer
    Dint(i32),
    /// atomic data type: LINT, 64-bit integer
    Lint(i64),
    /// atomic data type: REAL, 32-bit float
    Real(f32),
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
            Self::Bool(_) => TagType::Bool,
            Self::Dword(_) => TagType::Dword,
            Self::Sint(_) => TagType::Sint,
            Self::Int(_) => TagType::Int,
            Self::Dint(_) => TagType::Dint,
            Self::Lint(_) => TagType::Lint,
            Self::Real(_) => TagType::Real,
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
            0xC2 => TagValue::Sint(unsafe { mem::transmute(src[2]) }),
            0xC3 => TagValue::Int(LittleEndian::read_i16(&src[2..4])),
            0xC4 => {
                assert!(src.len() >= 6);
                TagValue::Dint(LittleEndian::read_i32(&src[2..6]))
            }
            0xCA => {
                assert!(src.len() >= 6);
                TagValue::Real(LittleEndian::read_f32(&src[2..6]))
            }
            0xD3 => {
                assert!(src.len() >= 6);
                TagValue::Dword(LittleEndian::read_u32(&src[2..6]))
            }
            0xC5 => {
                assert!(src.len() >= 10);
                TagValue::Lint(LittleEndian::read_i64(&src[2..10]))
            }
            0xC1 => TagValue::Bool(src[4] == 255),
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
            Self::Bool(v) => {
                dst.put_u16_le(0xC1);
                dst.put_slice(&[1, 0]);
                dst.put_u8(if v { 255 } else { 0 });
                dst.put_u8(0);
            }
            Self::Sint(v) => {
                dst.put_u16_le(0xC2);
                dst.put_slice(&[1, 0]);
                dst.put_i8(v);
                dst.put_u8(0);
            }
            Self::Int(v) => {
                dst.put_u16_le(0xC3);
                dst.put_slice(&[1, 0]);
                dst.put_i16_le(v);
            }
            Self::Dint(v) => {
                dst.put_u16_le(0xC4);
                dst.put_slice(&[1, 0]);
                dst.put_i32_le(v);
            }
            Self::Real(v) => {
                dst.put_u16_le(0xCA);
                dst.put_slice(&[1, 0]);
                dst.put_f32_le(v);
            }
            Self::Dword(v) => {
                dst.put_u16_le(0xD3);
                dst.put_slice(&[1, 0]);
                dst.put_u32_le(v);
            }
            Self::Lint(v) => {
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
            Self::Bool(_) => 6,
            Self::Sint(_) => 6,
            Self::Int(_) => 6,
            Self::Dint(_) => 8,
            Self::Real(_) => 8,
            Self::Dword(_) => 8,
            Self::Lint(_) => 12,
            Self::Structure { data, .. } => 4 + data.bytes_count(),
        }
    }
}
