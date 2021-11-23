pub use rseip_core::String;

/// only one service for ListServices
/// type code = 0x100
/// encoded bytes count: 24
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ListServiceItem {
    /// version shall be 1
    pub protocol_version: u16,
    pub capability: u16,
    /// name of service, NULL-terminated ASCII string;
    /// name = "Communications"
    pub name: String,
}

impl ListServiceItem {
    /// supports CIP Encapsulation via TCP
    #[inline(always)]
    pub fn capability_tcp(&self) -> bool {
        self.capability & 0b100000 > 0
    }

    /// support CIP Class 0 or 1 via UDP
    #[inline(always)]
    pub fn capability_udp(&self) -> bool {
        self.capability & 0b100000000 > 0
    }
}
