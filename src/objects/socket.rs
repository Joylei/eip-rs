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
