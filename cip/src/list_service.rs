// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

// type code = 0x100
// encoded bytes count: 24

use std::borrow::Cow;

/// only one service for ListServices
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ListServiceItem<'a> {
    /// version shall be 1
    pub protocol_version: u16,
    pub capability: u16,
    /// name of service, NULL-terminated ASCII string;
    /// it's Communications for CIP
    pub name: Cow<'a, str>,
}

impl ListServiceItem<'_> {
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
