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
    pub name: [u8; 16],
}

impl ListServiceItem {
    pub fn capability_cip(&self) -> bool {
        self.capability & 0b100000 > 0
    }

    pub fn capability_udp(&self) -> bool {
        self.capability & 0b100000000 > 0
    }
}
