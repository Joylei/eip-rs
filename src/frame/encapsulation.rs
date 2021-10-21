use bytes::Bytes;

/// UCMM: 504 bytes
/// max: 65535
#[derive(Debug, Default)]
pub struct EncapsulationPacket {
    pub hdr: EncapsulationHeader,
    /// max length: 65511
    pub data: Option<Bytes>,
}

/// header: 24 bytes
#[derive(Debug, Default)]
pub struct EncapsulationHeader {
    pub command: u16,
    /// Length, in bytes, of the data portion of the message
    pub length: u16,
    pub session_handler: u32,
    pub status: u32,
    pub sender_context: [u8; 8],
    /// shall be 0, receiver should ignore the command if not zero
    pub options: u32,
}
