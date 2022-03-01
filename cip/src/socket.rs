// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, Bytes};
use rseip_core::{
    codec::{Encode, Encoder},
    Error,
};

pub const AF_INET: i16 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SocketAddr {
    /// big-endian, shall be AF_INET=2
    pub sin_family: i16,
    /// big-endian
    pub sin_port: u16,
    /// big-endian
    pub sin_addr: u32,
    ///big-endian, shall be 0
    pub sin_zero: [u8; 8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    /// type id = 0x8000
    ToTarget,
    /// type id = 0x8001
    ToOriginator,
}

impl SocketType {
    /// CIP type id
    pub const fn type_id(&self) -> u16 {
        match self {
            Self::ToTarget => 0x8000,
            Self::ToOriginator => 0x8001,
        }
    }
}

impl SocketAddr {
    /// note unchecked
    #[inline]
    pub(crate) fn from_bytes<E: Error>(buf: Bytes) -> Result<Self, E> {
        let mut addr = SocketAddr {
            sin_family: BigEndian::read_i16(&buf[0..2]),
            sin_port: BigEndian::read_u16(&buf[2..4]),
            sin_addr: BigEndian::read_u32(&buf[4..8]),
            sin_zero: Default::default(),
        };
        addr.sin_zero.copy_from_slice(&buf[8..16]);
        Ok(addr)
    }
}

impl Encode for SocketAddr {
    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut bytes::BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error> {
        buf.put_i16(self.sin_family);
        buf.put_u16(self.sin_port);
        buf.put_u32(self.sin_addr);
        buf.put_slice(&self.sin_zero);
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        16
    }
}
