// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

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
    #[inline(always)]
    pub fn type_id(&self) -> u16 {
        match self {
            Self::ToTarget => 0x8000,
            Self::ToOriginator => 0x8001,
        }
    }
}
