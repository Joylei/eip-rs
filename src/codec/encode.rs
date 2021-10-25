mod cip;
mod command;
mod common_packet;
mod connected_send;
mod encapsulation;
mod epath;
mod message_request;
mod unconnected_send;

use super::{ClientCodec, Encodable};
use crate::{
    consts::ENCAPSULATION_DATA_MAX_LEN,
    error::Error,
    frame::{
        common_packet::{CommonPacketFormat, CommonPacketItem},
        encapsulation::EncapsulationPacket,
        Request,
    },
};
use bytes::{BufMut, Bytes, BytesMut};
use tokio_util::codec::Encoder;

impl Encodable for () {
    #[inline(always)]
    fn encode(self, _: &mut bytes::BytesMut) -> crate::Result<()> {
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        0
    }
}

impl<D1, D2> Encodable for (D1, D2)
where
    D1: Encodable,
    D2: Encodable,
{
    #[inline(always)]
    fn encode(self, dst: &mut bytes::BytesMut) -> crate::Result<()> {
        self.0.encode(dst)?;
        self.1.encode(dst)?;
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.0.bytes_count() + self.1.bytes_count()
    }
}

impl<D1, D2, D3> Encodable for (D1, D2, D3)
where
    D1: Encodable,
    D2: Encodable,
    D3: Encodable,
{
    #[inline(always)]
    fn encode(self, dst: &mut bytes::BytesMut) -> crate::Result<()> {
        self.0.encode(dst)?;
        self.1.encode(dst)?;
        self.2.encode(dst)?;
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.0.bytes_count() + self.1.bytes_count() + self.2.bytes_count()
    }
}

impl Encodable for &[u8] {
    #[inline(always)]
    fn encode(self, dst: &mut bytes::BytesMut) -> crate::Result<()> {
        dst.put_slice(self);
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.len()
    }
}

pub struct LazyEncode<F> {
    pub f: F,
    pub bytes_count: usize,
}

impl<F> Encodable for LazyEncode<F>
where
    F: FnOnce(&mut bytes::BytesMut) -> crate::Result<()> + Send,
{
    #[inline(always)]
    fn encode(self, dst: &mut bytes::BytesMut) -> crate::Result<()> {
        (self.f)(dst)
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.bytes_count
    }
}

impl<E: Encodable> Encoder<E> for ClientCodec {
    type Error = Error;
    #[inline(always)]
    fn encode(&mut self, item: E, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        item.encode(dst)
    }
}

impl Encoder<Request> for ClientCodec {
    type Error = Error;
    fn encode(&mut self, item: Request, dst: &mut BytesMut) -> Result<(), Self::Error> {
        use crate::frame::Request::*;
        let mut pkt = EncapsulationPacket::default();
        pkt.hdr.command = item.command_code();

        match item {
            Nop { data } => {
                if let Some(ref data) = data {
                    debug_assert!(data.len() <= ENCAPSULATION_DATA_MAX_LEN);
                }
                pkt.data = data;
            }
            RegisterSession { sender_context } => {
                pkt.hdr.sender_context.copy_from_slice(&sender_context);

                let mut data = BytesMut::with_capacity(4);
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
                pkt.hdr.session_handle = session_handle;
                pkt.hdr.sender_context.copy_from_slice(&sender_context);
            }
            ListServices { sender_context } => {
                pkt.hdr.sender_context.copy_from_slice(&sender_context);
            }
            SendRRData {
                session_handle,
                timeout,
                data,
            } => {
                pkt.hdr.session_handle = session_handle;
                let mut buf = BytesMut::new();
                buf.put_u32_le(0); // interface handle, shall be 0 for CIP
                buf.put_u16_le(timeout);
                //cpf
                let cpf = CommonPacketFormat::from(vec![
                    CommonPacketItem::with_null_addr(),
                    CommonPacketItem::with_connected_data(data.unwrap()),
                ]);

                self.encode(cpf, dst)?;

                pkt.data = Some(buf.freeze());
            }
            SendUnitData {
                session_handle,
                connection_id,
                sequence_number,
                data,
            } => {
                pkt.hdr.session_handle = session_handle;
                let mut buf = BytesMut::new();
                buf.put_u32_le(0); // interface handle, shall be 0 for CIP
                buf.put_u16_le(0); // timeout, 0 for SendUnitData

                let addr_item = {
                    let mut buf = BytesMut::new();
                    buf.put_u32_le(connection_id);
                    if let Some(sid) = sequence_number {
                        buf.put_u32_le(sid);
                        CommonPacketItem {
                            type_code: 0x8002, // sequenced address item
                            data: Some(buf.freeze()),
                        }
                    } else {
                        CommonPacketItem {
                            type_code: 0xA1, // connected address item
                            data: Some(buf.freeze()),
                        }
                    }
                };

                //cpf
                let cpf = CommonPacketFormat::from(vec![
                    addr_item,
                    CommonPacketItem::with_unconnected_data(data.unwrap()),
                ]);

                self.encode(cpf, dst)?;

                pkt.data = Some(buf.freeze());
            }
            ListIdentity | ListInterfaces => {}
            _ => unimplemented!(),
        }
        self.encode(pkt, dst)
    }
}

impl Encodable for Bytes {
    #[inline(always)]
    fn encode(self, dst: &mut bytes::BytesMut) -> Result<(), Error> {
        dst.put_slice(&self);
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.len()
    }
}

impl<D: Encodable> Encodable for Option<D> {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> crate::Result<()> {
        if let Some(item) = self {
            item.encode(dst)
        } else {
            Ok(())
        }
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        self.as_ref().map(|v| v.bytes_count()).unwrap_or_default()
    }
}
