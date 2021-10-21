/// default port for EtherNet/IP over TCP/IP
pub const ENIP_DEFAULT_PORT: u32 = 0xAF12;
/// default port for EtherNet/IP over TCP/IP class 0 and class 1
pub const ENIP_DEFAULT_UDP_PORT: u32 = 0x08AE;

pub(crate) const ENCAPSULATION_HEADER_LEN: usize = 24;
pub(crate) const ENCAPSULATION_DATA_MAX_LEN: usize = u16::MAX as usize - ENCAPSULATION_HEADER_LEN;
