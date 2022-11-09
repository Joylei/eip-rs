// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::ClientError;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::marker::PhantomData;
use rseip_core::{codec::*, Error};
use smallvec::SmallVec;

/// atomic data type: BOOL
#[allow(clippy::upper_case_acronyms)]
#[allow(unused)]
pub type BOOL = bool;
/// atomic data type: DWORD, 32-bit boolean array
#[allow(clippy::upper_case_acronyms)]
#[allow(unused)]
pub type DWORD = u32;
/// atomic data type: SINT, 8-bit integer
#[allow(clippy::upper_case_acronyms)]
#[allow(unused)]
pub type SINT = i8;
/// atomic data type: INT, 16-bit integer
#[allow(clippy::upper_case_acronyms)]
#[allow(unused)]
pub type INT = i16;
/// atomic data type: DINT, 32-bit integer
#[allow(clippy::upper_case_acronyms)]
#[allow(unused)]
pub type DINT = i32;
/// atomic data type: LINT, 64-bit integer
#[allow(clippy::upper_case_acronyms)]
#[allow(unused)]
pub type LINT = i64;
/// atomic data type: REAL, 32-bit float
#[allow(clippy::upper_case_acronyms)]
#[allow(unused)]
pub type REAL = f32;

/// tag type for AB PLC
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl Encode for TagType {
    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        match self {
            Self::Bool => {
                encoder.encode_u16(0xC1, buf)?;
            }
            Self::Sint => {
                encoder.encode_u16(0xC2, buf)?;
            }
            Self::Int => {
                encoder.encode_u16(0xC3, buf)?;
            }
            Self::Dint => {
                encoder.encode_u16(0xC4, buf)?;
            }
            Self::Real => {
                encoder.encode_u16(0xCA, buf)?;
            }
            Self::Dword => {
                encoder.encode_u16(0xD3, buf)?;
            }
            Self::Lint => {
                encoder.encode_u16(0xC5, buf)?;
            }
            Self::Structure(handle) => {
                encoder.encode(&[0xA0, 0x02], buf)?;
                encoder.encode_u16(*handle, buf)?;
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        match self {
            Self::Structure(_) => 4,
            _ => 2,
        }
    }
}

impl<'de> Decode<'de> for TagType {
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.ensure_size(3)?;
        let type_code = decoder.decode_u16();
        let val = match type_code {
            0xC2 => TagType::Sint,
            0xC3 => TagType::Int,
            0xC4 => TagType::Dint,
            0xCA => TagType::Real,
            0xD3 => TagType::Dword,
            0xC5 => TagType::Lint,
            0xC1 => TagType::Bool,
            0x02A0 => {
                decoder.ensure_size(2)?;
                TagType::Structure(decoder.decode_u16())
            }
            _ => {
                return Err(Error::custom(format!(
                    "unexpected type code: {}",
                    type_code
                )))
            }
        };
        Ok(val)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagValue<V> {
    pub tag_type: TagType,
    pub value: V,
}

macro_rules! impl_atomic {
    ($ty:ty, $size: tt) => {
        impl<'de> Decode<'de> for TagValue<$ty> {
            #[inline]
            fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                let tag_type = decoder.decode_any()?;
                let value = decoder.decode_any()?;
                Ok(Self { tag_type, value })
            }
        }

        impl Encode for TagValue<$ty> {
            #[inline]
            fn encode<A: Encoder>(
                self,
                buf: &mut BytesMut,
                encoder: &mut A,
            ) -> Result<(), A::Error> {
                self.tag_type.encode(buf, encoder)?;
                buf.put_slice(&[1, 0]);
                self.value.encode(buf, encoder)?;
                Ok(())
            }
            #[inline]
            fn encode_by_ref<A: Encoder>(
                &self,
                buf: &mut BytesMut,
                encoder: &mut A,
            ) -> Result<(), A::Error> {
                self.tag_type.encode(buf, encoder)?;
                buf.put_slice(&[1, 0]);
                self.value.encode_by_ref(buf, encoder)?;
                Ok(())
            }

            #[inline]
            fn bytes_count(&self) -> usize {
                self.tag_type.bytes_count() + 2 + $size
            }
        }
    };
}

impl_atomic!(bool, 1);
impl_atomic!(i8, 1);
impl_atomic!(u8, 1);
impl_atomic!(i16, 2);
impl_atomic!(u16, 2);
impl_atomic!(i32, 4);
impl_atomic!(u32, 4);
impl_atomic!(i64, 8);
impl_atomic!(u64, 8);
impl_atomic!(f32, 4);
impl_atomic!(f64, 8);

macro_rules! impl_seq {
    ($ty:tt) => {
        impl<T> Encode for TagValue<$ty<T>>
        where
            T: Encode,
        {
            #[inline]
            fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
            where
                Self: Sized,
            {
                self.tag_type.encode(buf, encoder)?;
                encoder.encode_u16(self.value.len() as u16, buf)?;
                self.value.encode(buf, encoder)?;
                Ok(())
            }

            #[inline]
            fn encode_by_ref<A: Encoder>(
                &self,
                buf: &mut BytesMut,
                encoder: &mut A,
            ) -> Result<(), A::Error> {
                self.tag_type.encode(buf, encoder)?;
                encoder.encode_u16(self.value.len() as u16, buf)?;
                self.value.encode_by_ref(buf, encoder)?;
                Ok(())
            }

            #[inline]
            fn bytes_count(&self) -> usize {
                self.tag_type.bytes_count() + 2 + self.value.bytes_count()
            }
        }
    };
}

impl_seq!(Vec);

impl<T, const N: usize> Encode for TagValue<[T; N]>
where
    T: Encode,
{
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        self.tag_type.encode(buf, encoder)?;
        encoder.encode_u16(self.value.len() as u16, buf)?;
        self.value.encode(buf, encoder)?;
        Ok(())
    }

    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        self.tag_type.encode(buf, encoder)?;
        encoder.encode_u16(self.value.len() as u16, buf)?;
        self.value.encode_by_ref(buf, encoder)?;
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        self.tag_type.bytes_count() + 2 + self.value.bytes_count()
    }
}

impl<T> Encode for TagValue<&[T]>
where
    T: Encode,
{
    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        self.tag_type.encode(buf, encoder)?;
        encoder.encode_u16(self.value.len() as u16, buf)?;
        for item in self.value.iter() {
            item.encode_by_ref(buf, encoder)?;
        }
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        self.tag_type.bytes_count() + 2 + self.value.iter().map(|v| v.bytes_count()).sum::<usize>()
    }
}

impl<'de> Decode<'de> for TagValue<Bytes> {
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let tag_type: TagType = decoder.decode_any()?;
        let data: BytesHolder = decoder.decode_any()?;
        Ok(Self {
            tag_type,
            value: data.into(),
        })
    }
}

impl<'de, T> Decode<'de> for TagValue<Vec<T>>
where
    T: Decode<'de>,
{
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let tag_type: TagType = decoder.decode_any()?;
        let mut data = Vec::new();
        while decoder.buf().has_remaining() {
            let v: T = decoder.decode_any()?;
            data.push(v);
        }
        Ok(Self {
            tag_type,
            value: data,
        })
    }
}

impl<'de, T> Decode<'de> for TagValue<SmallVec<T>>
where
    T: smallvec::Array,
    T::Item: Decode<'de>,
{
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let tag_type: TagType = decoder.decode_any()?;
        let mut data = SmallVec::new();
        while decoder.buf().has_remaining() {
            let v: T::Item = decoder.decode_any()?;
            data.push(v);
        }
        Ok(Self {
            tag_type,
            value: data,
        })
    }
}

#[derive(Debug)]
pub struct TagValueTypedIter<T> {
    tag_type: TagType,
    decoder: LittleEndianDecoder<ClientError>,
    _marker: PhantomData<T>,
}

impl<T> TagValueTypedIter<T> {
    #[allow(unused)]
    pub fn tag_type(&self) -> TagType {
        self.tag_type
    }

    pub fn from_bytes(buf: Bytes) -> Result<Self, ClientError> {
        let mut decoder = LittleEndianDecoder::new(buf);
        let tag_type = decoder.decode_any()?;
        Ok(Self {
            tag_type,
            decoder,
            _marker: Default::default(),
        })
    }
}

impl<'de, T> Iterator for TagValueTypedIter<T>
where
    T: Decode<'de>,
{
    type Item = Result<T, ClientError>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.decoder.has_remaining() {
            Some(self.decoder.decode_any())
        } else {
            None
        }
    }
}

impl<'de, T> Decode<'de> for TagValueTypedIter<T> {
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let tag_type: TagType = decoder.decode_any()?;
        let len = decoder.buf_mut().remaining();
        let buf = decoder.buf_mut().copy_to_bytes(len);
        let res = Self {
            tag_type,
            decoder: LittleEndianDecoder::new(buf),
            _marker: Default::default(),
        };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct TagValueIter {
    tag_type: TagType,
    decoder: LittleEndianDecoder<ClientError>,
}

impl TagValueIter {
    #[allow(unused)]
    pub fn tag_type(&self) -> TagType {
        self.tag_type
    }

    pub fn from_bytes(buf: Bytes) -> Result<Self, ClientError> {
        let mut decoder = LittleEndianDecoder::new(buf);
        let tag_type = decoder.decode_any()?;
        Ok(Self { tag_type, decoder })
    }
}

impl TagValueIter {
    #[inline]
    pub fn next<'de, T>(&mut self) -> Option<Result<T, ClientError>>
    where
        T: Decode<'de>,
    {
        if self.decoder.has_remaining() {
            Some(self.decoder.decode_any())
        } else {
            None
        }
    }
}

impl<'de> Decode<'de> for TagValueIter {
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let tag_type: TagType = decoder.decode_any()?;
        let len = decoder.buf_mut().remaining();
        let buf = decoder.buf_mut().copy_to_bytes(len);
        let res = Self {
            tag_type,
            decoder: LittleEndianDecoder::new(buf),
        };
        Ok(res)
    }
}
