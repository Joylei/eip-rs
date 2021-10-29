use crate::{Error, Result};

// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2020-2021, Joylei <leingliu@gmail.com>
// License: MIT

/// UCMM: 504 bytes
/// max: 65535
#[derive(Debug, Default)]
pub struct EncapsulationPacket<D> {
    pub hdr: EncapsulationHeader,
    /// max length: 65511
    pub data: D,
}

/// header: 24 bytes
#[derive(Debug, Default)]
pub struct EncapsulationHeader {
    pub command: u16,
    /// Length, in bytes, of the data portion of the message
    pub length: u16,
    pub session_handle: u32,
    pub status: u32,
    pub sender_context: [u8; 8],
    /// shall be 0, receiver should ignore the command if not zero
    pub options: u32,
}

impl EncapsulationHeader {
    #[inline(always)]
    pub fn ensure_command(&self, command_code: u16) -> Result<()> {
        if self.command != command_code {
            return Err(Error::InvalidCommandReply {
                expect: command_code,
                actual: self.command,
            });
        }
        Ok(())
    }
}
