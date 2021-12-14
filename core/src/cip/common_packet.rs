// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{codec::*, hex::AsHex, Error};
use bytes::{Buf, Bytes, BytesMut};
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use smallvec::SmallVec;

type Array<T> = [CommonPacketItem<T>; 4];

/// common packet format
#[derive(Default, Debug)]
pub struct CommonPacket<T>(SmallVec<Array<T>>);

impl<T> CommonPacket<T> {
    /// new object
    #[inline]
    pub fn new() -> Self {
        Self(Default::default())
    }

    /// append an item
    #[inline]
    pub fn push(&mut self, item: CommonPacketItem<T>) {
        self.0.push(item);
    }

    /// panic if idx is out of range
    #[inline]
    pub fn remove(&mut self, idx: usize) -> CommonPacketItem<T> {
        self.0.remove(idx)
    }
}

impl<T> Deref for CommonPacket<T> {
    type Target = [CommonPacketItem<T>];
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for CommonPacket<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<Vec<CommonPacketItem<T>>> for CommonPacket<T> {
    #[inline]
    fn from(src: Vec<CommonPacketItem<T>>) -> Self {
        Self(SmallVec::from_vec(src))
    }
}

impl<T> IntoIterator for CommonPacket<T> {
    type Item = CommonPacketItem<T>;
    type IntoIter = crate::iter::IntoIter<Array<T>>;
    fn into_iter(self) -> Self::IntoIter {
        crate::iter::IntoIter::new(self.0)
    }
}

/// common packet format item
#[derive(Debug)]
pub struct CommonPacketItem<T> {
    pub type_code: u16,
    pub data: T,
}

impl CommonPacketItem<Bytes> {
    /// null address
    #[inline]
    pub fn with_null_addr() -> Self {
        Self {
            type_code: 0,
            data: Bytes::from_static(&[0x00, 0x00]),
        }
    }

    /// unconnected data item
    #[inline]
    pub fn with_unconnected_data(data: Bytes) -> Self {
        Self {
            type_code: 0xB2,
            data,
        }
    }

    /// connected data item
    #[inline]
    pub fn with_connected_data(data: Bytes) -> Self {
        Self {
            type_code: 0xB1,
            data,
        }
    }

    /// is null address
    #[inline]
    pub fn is_null_addr(&self) -> bool {
        if self.type_code != 0 {
            return false;
        }
        self.data.is_empty()
    }
}

impl<T> CommonPacketItem<T> {
    /// ensure current item matches the specified type code
    #[inline]
    pub fn ensure_type_code<E: Error>(&self, type_code: u16) -> Result<(), E> {
        if self.type_code != type_code {
            return Err(E::invalid_value(
                format_args!("common packet item type {}", self.type_code.as_hex()),
                type_code.as_hex(),
            ));
        }
        Ok(())
    }
}

/// lazy decoder for common packet
pub struct CommonPacketIter<'de, D> {
    pub(crate) decoder: D,
    offset: u16,
    total: u16,
    _marker: PhantomData<&'de D>,
}

impl<'de, D> CommonPacketIter<'de, D>
where
    D: Decoder<'de>,
{
    #[inline]
    pub fn new(mut decoder: D) -> Result<Self, D::Error> {
        let item_count: u16 = decoder.decode_any()?;
        Ok(Self {
            decoder,
            total: item_count,
            offset: 0,
            _marker: Default::default(),
        })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.total as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.total == 0
    }

    #[inline]
    fn has_remaining(&self) -> bool {
        !(self.total == 0 || self.offset >= self.total)
    }

    /// next common packet item, strong typed
    #[inline]
    pub fn next_typed<T>(&mut self) -> Option<Result<CommonPacketItem<T>, D::Error>>
    where
        T: Decode<'de> + 'static,
    {
        self.move_next()
    }

    /// next common packet item
    #[inline]
    pub fn next_item(&mut self) -> Option<Result<CommonPacketItem<Bytes>, D::Error>> {
        self.move_next()
    }

    /// next common packet item
    #[inline]
    fn move_next<T>(&mut self) -> Option<Result<T, D::Error>>
    where
        T: Decode<'de>,
    {
        if !self.has_remaining() {
            return None;
        }

        let res = self.decoder.decode_any();
        if res.is_ok() {
            self.offset += 1;
        }
        Some(res)
    }

    /// visit next common packet item
    #[inline]
    pub fn accept<V>(
        &mut self,
        expected_type_code: u16,
        visitor: V,
    ) -> Option<Result<CommonPacketItem<V::Value>, D::Error>>
    where
        V: Visitor<'de>,
    {
        if !self.has_remaining() {
            return None;
        }
        let res =
            CommonPacketItem::validate_and_decode(&mut self.decoder, expected_type_code, visitor);
        if res.is_ok() {
            self.offset += 1;
        }
        Some(res)
    }
}

impl<T: Encode> Encode for CommonPacket<T> {
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        debug_assert!(self.len() <= u16::MAX as usize);
        encoder.encode(self.len() as u16, buf)?;
        for item in self.into_iter() {
            encoder.encode(item, buf)?;
        }
        Ok(())
    }
    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        debug_assert!(self.len() <= u16::MAX as usize);
        encoder.encode(self.len() as u16, buf)?;
        for item in self.0.iter() {
            encoder.encode_by_ref(item, buf)?;
        }
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        let count: usize = self.iter().map(|v| v.bytes_count()).sum();
        count + 2
    }
}

impl<T> CommonPacketItem<T> {
    pub fn validate_and_decode<'de, D, V>(
        mut decoder: D,
        expected_type_code: u16,
        visitor: V,
    ) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
        V: Visitor<'de, Value = T>,
    {
        decoder.ensure_size(4)?;
        let type_code = decoder.decode_u16();
        if type_code != expected_type_code {
            return Err(Error::invalid_value(
                format_args!("common packet type code {:#02x}", type_code),
                expected_type_code.as_hex(),
            ));
        }
        let item_length = decoder.decode_u16();
        let data = decoder.decode_sized(item_length as usize, visitor)?;
        Ok(Self { type_code, data })
    }

    pub fn decode_with<'de, D, V>(mut decoder: D, visitor: V) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
        V: Visitor<'de, Value = T>,
    {
        decoder.ensure_size(4)?;
        let type_code = decoder.decode_u16();
        let item_length = decoder.decode_u16();
        let data = decoder.decode_sized(item_length as usize, visitor)?;
        Ok(Self { type_code, data })
    }
}

impl<'de, T: Decode<'de> + 'static> Decode<'de> for CommonPacket<T> {
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let item_count: u16 = decoder.decode_any()?;
        let mut cpf = CommonPacket::new();
        for _ in 0..item_count {
            cpf.push(decoder.decode_any()?);
        }
        Ok(cpf)
    }
}

impl<'de> Decode<'de> for CommonPacketItem<Bytes> {
    #[inline]
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.ensure_size(4)?;
        let type_code = decoder.decode_u16();
        let item_length = decoder.decode_u16() as usize;
        decoder.ensure_size(item_length)?;
        Ok(Self {
            type_code,
            data: decoder.buf_mut().copy_to_bytes(item_length),
        })
    }
}

impl<'de, T: Decode<'de> + 'static> Decode<'de> for CommonPacketItem<T> {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Self::decode_with(decoder, visitor::any())
    }
}

impl<T: Encode> Encode for CommonPacketItem<T> {
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error> {
        debug_assert!(self.bytes_count() <= u16::MAX as usize);
        encoder.encode_u16(self.type_code, buf)?;
        encoder.encode_u16(self.data.bytes_count() as u16, buf)?;
        encoder.encode(self.data, buf)?;
        Ok(())
    }
    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        debug_assert!(self.bytes_count() <= u16::MAX as usize);
        encoder.encode_u16(self.type_code, buf)?;
        encoder.encode_u16(self.data.bytes_count() as u16, buf)?;
        encoder.encode_by_ref(&self.data, buf)?;
        Ok(())
    }
    #[inline]
    fn bytes_count(&self) -> usize {
        4 + self.data.bytes_count()
    }
}
