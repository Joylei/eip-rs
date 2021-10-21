pub mod common_packet;
pub mod encapsulation;

use crate::objects::{identity::IdentityObject, service::ListServiceItem};
use bytes::Bytes;
pub use common_packet::{CommonPacketFormat, CommonPacketItem};
pub use encapsulation::{EncapsulationHeader, EncapsulationPacket};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// sent by either end, no reply
    Nop {
        data: Option<Bytes>,
    },
    ListIdentity,
    ListInterfaces,
    ListServices {
        sender_context: Bytes,
    },
    RegisterSession {
        sender_context: Bytes,
    },
    UnRegisterSession {
        session_handle: u32,
        sender_context: Bytes,
    },
    /// for UCMM (unconnected message), sent by originator
    SendRRData {
        /// shall be 0 for CIP
        interface_handle: u32,
        /// operation timeout, in seconds;
        /// - set to 0, rely on the timeout mechanism of the encapsulated protocol
        /// - usually set to 0 for CIP
        timeout: u16,
        /// encoded in Common Packet Format
        cpf: Option<Bytes>,
    },
    /// for connected message, sent by either end, no reply
    SendUnitData,
    IndicateStatus,
    Cancel,
}

impl Request {
    pub(crate) fn command_code(&self) -> u16 {
        match self {
            Self::Nop { .. } => 0x0000,
            Self::ListServices { .. } => 0x0004,
            Self::ListIdentity => 0x0063,
            Self::ListInterfaces => 0x0064,
            Self::RegisterSession { .. } => 0x0065,
            Self::UnRegisterSession { .. } => 0x0066,
            Self::SendRRData { .. } => 0x006F,
            Self::SendUnitData => 0x0070,
            Self::IndicateStatus => 0x0072,
            Self::Cancel => 0x0073,
        }
    }
}

#[derive(Debug)]
pub enum Response {
    ListServices(Vec<ListServiceItem>),
    ListIdentity(Vec<IdentityObject>),
    ListInterfaces,
    RegisterSession {
        session_handle: u32,
        protocol_version: u16,
    },
    SendRRData(CommonPacketFormat),
    IndicateStatus,
    Cancel,
}
