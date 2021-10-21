mod bytes_count;
pub mod decode;
pub mod encode;

use crate::{
    consts::ENCAPSULATION_DATA_MAX_LEN,
    frame::{encapsulation::EncapsulationPacket, Request},
};
use bytes::{BufMut, BytesMut};

/// encoded bytes count
pub trait EncodedBytesCount {
    /// encoded bytes count
    fn bytes_count(&self) -> usize;
}

#[derive(Debug, PartialEq)]
pub struct EIPDecoder;

#[derive(Debug, PartialEq)]
pub struct ClientCodec {
    pub(crate) decoder: EIPDecoder,
}

impl Default for ClientCodec {
    fn default() -> Self {
        Self {
            decoder: EIPDecoder,
        }
    }
}

impl From<Request> for EncapsulationPacket {
    fn from(req: Request) -> Self {
        use crate::frame::Request::*;
        let mut pkt: Self = Default::default();
        pkt.hdr.command = req.command_code();

        match req {
            Nop { data } => {
                if let Some(ref data) = data {
                    debug_assert!(data.len() <= ENCAPSULATION_DATA_MAX_LEN);
                }
                pkt.data = data;
            }
            RegisterSession { sender_context } => {
                pkt.hdr.sender_context.copy_from_slice(&sender_context);

                let mut data = BytesMut::with_capacity(2);
                // protocol version, shall be 1
                data.put_u16_le(1);
                // session options, shall be 0
                data.put_u16_le(0);

                pkt.data = Some(data.freeze());
            }
            UnRegisterSession {
                session_handle,
                sender_context,
            } => {
                pkt.hdr.session_handler = session_handle;
                pkt.hdr.sender_context.copy_from_slice(&sender_context);
            }
            ListServices { sender_context } => {
                pkt.hdr.sender_context.copy_from_slice(&sender_context);
            }
            SendRRData {
                interface_handle,
                timeout,
                cpf,
            } => {
                let cpf_len = cpf.as_ref().map(|v| v.len()).unwrap_or_default();
                let mut data = BytesMut::with_capacity(6 + cpf_len);
                data.put_u32_le(interface_handle);
                data.put_u16_le(timeout);
                if let Some(cpf) = cpf {
                    data.put_slice(&cpf);
                }
                debug_assert!(data.len() <= ENCAPSULATION_DATA_MAX_LEN);
                pkt.data = Some(data.freeze());
            }
            ListIdentity | ListInterfaces => {}
            _ => unimplemented!(),
        }
        pkt
    }
}
