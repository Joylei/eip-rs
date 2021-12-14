// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

/// parameters for Unconnected Send
#[derive(Debug)]
pub struct UnconnectedSend<P, D> {
    pub priority_ticks: u8,
    pub timeout_ticks: u8,
    /// connection path
    pub path: P,
    /// request data
    pub data: D,
}

impl<P, D> UnconnectedSend<P, D> {
    #[inline]
    pub fn new(path: P, data: D) -> Self {
        Self {
            priority_ticks: 0x03,
            timeout_ticks: 0xFA,
            path,
            data,
        }
    }
}
