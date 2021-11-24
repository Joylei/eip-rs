// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use core::{
    convert::TryFrom,
    ops::{Deref, DerefMut},
};
use smallvec::SmallVec;
use std::io::{self, Error, ErrorKind};

/// common packet format
#[derive(Default, Debug)]
pub struct CommonPacket(SmallVec<[CommonPacketItem; 2]>);

impl CommonPacket {
    #[inline(always)]
    pub fn new() -> Self {
        Self(Default::default())
    }

    #[inline(always)]
    pub fn into_inner(self) -> SmallVec<[CommonPacketItem; 2]> {
        self.0
    }

    #[inline(always)]
    pub fn into_iter(self) -> impl IntoIterator<Item = CommonPacketItem> {
        self.0.into_iter()
    }

    #[inline(always)]
    pub fn push(&mut self, item: CommonPacketItem) {
        self.0.push(item);
    }
}

impl Deref for CommonPacket {
    type Target = [CommonPacketItem];
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CommonPacket {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<CommonPacketItem>> for CommonPacket {
    #[inline(always)]
    fn from(src: Vec<CommonPacketItem>) -> Self {
        Self(SmallVec::from_vec(src))
    }
}

/// common packet format item
#[derive(Debug)]
pub struct CommonPacketItem {
    pub type_code: u16,
    pub data: Bytes,
}

impl CommonPacketItem {
    /// null address
    #[inline(always)]
    pub fn with_null_addr() -> Self {
        Self {
            type_code: 0,
            data: Bytes::from_static(&[0x00, 0x00]),
        }
    }

    /// unconnected data item
    #[inline(always)]
    pub fn with_unconnected_data(data: Bytes) -> Self {
        Self {
            type_code: 0xB2,
            data: data,
        }
    }

    /// connected data item
    #[inline(always)]
    pub fn with_connected_data(data: Bytes) -> Self {
        Self {
            type_code: 0xB1,
            data: data,
        }
    }

    /// is null address
    #[inline(always)]
    pub fn is_null_addr(&self) -> bool {
        if self.type_code != 0 {
            return false;
        }
        self.data.len() == 0
    }

    /// ensure current item matches the specified type code
    #[inline(always)]
    pub fn ensure_type_code(&self, type_code: u16) -> io::Result<()> {
        if self.type_code != type_code {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "common packet format item: unexpected type code - {:#0x}",
                    type_code
                ),
            ));
        }
        Ok(())
    }
}

impl TryFrom<Bytes> for CommonPacket {
    type Error = Error;
    #[inline]
    fn try_from(mut buf: Bytes) -> Result<Self, Self::Error> {
        if buf.len() < 2 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "common packet format: invalid data",
            ));
        }
        let item_count = LittleEndian::read_u16(&buf[0..2]);
        buf = buf.slice(2..);
        let mut cpf = CommonPacket::new();
        for _ in 0..item_count {
            if buf.len() < 4 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "common packet format: invalid data",
                ));
            }
            let type_code = LittleEndian::read_u16(&buf[0..2]);
            let item_length = LittleEndian::read_u16(&buf[2..4]) as usize;
            if buf.len() < 4 + item_length {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "common packet format: invalid data",
                ));
            }
            let item_data = buf.slice(4..4 + item_length);
            cpf.push(CommonPacketItem {
                type_code,
                data: item_data,
            });
            buf = buf.slice(4 + item_length..);
        }

        // should no remaining left
        if buf.len() != 0 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "common packet format: invalid data",
            ));
        }
        Ok(cpf)
    }
}
