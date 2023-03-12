// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::{
    //command::{self, Command},
    framed::Framed,
    EncapsulationPacket,
};
use crate::{codec::ClientCodec, consts::*, *};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, Bytes, BytesMut};
use core::fmt;
use futures_util::{AsyncRead, AsyncWrite};
use futures_util::{SinkExt, StreamExt};
use rseip_core::{
    cip::CommonPacketIter,
    codec::{Encode, LittleEndianDecoder},
};

pub type CommonPacket<'a, E> = CommonPacketIter<'a, LittleEndianDecoder<E>>;

/// EIP context
pub struct EipContext<T, E: Error> {
    framed: Framed<T, ClientCodec<E>>,
    session_handle: u32,
    #[allow(unused)]
    sender_context: Bytes,
}

impl<T, E: Error> fmt::Debug for EipContext<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EipContext")
            .field("session_handle", &self.session_handle)
            .field("sender_context", &self.sender_context)
            .field("framed", &"<Framed>")
            .finish()
    }
}

impl<T, E: Error> EipContext<T, E> {
    /// set sender context
    #[allow(unused)]
    #[inline]
    pub fn with_sender_context(&mut self, sender_context: [u8; 8]) -> &mut Self {
        let mut buf = BytesMut::new();
        buf.put_slice(&sender_context);
        self.sender_context = buf.freeze();
        self
    }

    /// current session handle
    #[inline]
    pub fn session_handle(&self) -> Option<u32> {
        if self.session_handle > 0 {
            Some(self.session_handle)
        } else {
            None
        }
    }

    /// session registered?
    #[inline]
    pub fn has_session(&self) -> bool {
        self.session_handle > 0
    }
}

impl<T, E> EipContext<T, E>
where
    T: AsyncRead + AsyncWrite + Unpin,
    E: Error + 'static,
{
    /// create [`EipContext`]
    #[inline]
    pub fn new(transport: T) -> Self {
        let framed = Framed::new(transport, ClientCodec::new());

        Self {
            framed,
            session_handle: 0,
            sender_context: Bytes::from_static(&[0, 0, 0, 0, 0, 0, 0, 0]),
        }
    }

    /// send and wait for reply
    #[inline]
    async fn send_and_reply(
        &mut self,
        pkt: EncapsulationPacket<Bytes>,
    ) -> Result<EncapsulationPacket<Bytes>, E> {
        let code = pkt.hdr.command;
        self.framed.send(pkt).await?;
        match self.framed.next().await {
            Some(item) => {
                let pkt: EncapsulationPacket<Bytes> = item?;
                pkt.hdr.ensure_command::<E>(code)?;
                Ok(pkt)
            }
            None => Err(E::custom("transport closed")),
        }
    }

    /// send command: NOP
    #[inline]
    pub async fn nop<D: Encode>(&mut self, data: D) -> Result<(), E> {
        log::trace!("send command: NOP");
        let mut buf = BytesMut::new();
        data.encode(&mut buf, self.framed.codec_mut())?;
        let mut pkt = EncapsulationPacket {
            hdr: Default::default(),
            data: buf.freeze(),
        };
        pkt.hdr.command = EIP_COMMAND_NOP;
        self.framed.send(pkt).await?;
        Ok(())
    }

    /// send command: ListIdentity
    #[allow(unused)]
    #[inline]
    pub async fn list_identity<'de>(&mut self) -> Result<CommonPacket<'static, E>, E> {
        let mut pkt = EncapsulationPacket::default();
        pkt.hdr.command = EIP_COMMAND_LIST_IDENTITY;
        let pkt = self.send_and_reply(pkt).await?;
        CommonPacket::new(LittleEndianDecoder::<E>::new(pkt.data))
    }

    /// send command: ListServices
    #[allow(unused)]
    #[inline]
    pub async fn list_service<'de>(&mut self) -> Result<CommonPacket<'static, E>, E> {
        let mut pkt = EncapsulationPacket::default();
        pkt.hdr.command = EIP_COMMAND_LIST_SERVICE;
        let pkt = self.send_and_reply(pkt).await?;
        CommonPacket::new(LittleEndianDecoder::<E>::new(pkt.data))
    }

    /// send command: ListInterface
    #[allow(unused)]
    #[inline]
    pub async fn list_interface<'de>(&mut self) -> Result<CommonPacket<'static, E>, E> {
        let mut pkt = EncapsulationPacket::default();
        pkt.hdr.command = EIP_COMMAND_LIST_INTERFACES;
        let pkt = self.send_and_reply(pkt).await?;
        CommonPacket::new(LittleEndianDecoder::<E>::new(pkt.data))
    }

    /// send command: RegisterSession
    #[inline]
    pub async fn register_session(&mut self) -> Result<u32, E> {
        if self.has_session() {
            return Err(E::custom("already has a session"));
        }
        let mut pkt = EncapsulationPacket::default();
        pkt.hdr.command = EIP_COMMAND_REGISTER_SESSION;
        pkt.data = Bytes::from_static(&[0x01, 0x00, 0x00, 0x00]);
        let pkt = self.send_and_reply(pkt).await?;

        let session_handle = pkt.hdr.session_handle;
        let reply_data = pkt.data;
        if reply_data.len() != 4 {
            return Err(E::invalid_length(reply_data.len(), 4));
        }
        #[cfg(debug_assertions)]
        {
            let protocol_version = LittleEndian::read_u16(&reply_data[0..2]);
            debug_assert_eq!(protocol_version, 1);
            let session_options = LittleEndian::read_u16(&reply_data[2..4]);
            debug_assert_eq!(session_options, 0);
        }
        if session_handle == 0 {
            return Err(E::invalid_value("session handle 0", ">0"));
        }
        self.session_handle = session_handle;
        Ok(session_handle)
    }

    /// send command: UnRegisterSession
    #[inline]
    pub async fn unregister_session(&mut self) -> Result<(), E> {
        if self.session_handle == 0 {
            return Ok(());
        }
        log::trace!("send command: UnRegisterSession");
        let mut pkt = EncapsulationPacket::default();
        pkt.hdr.command = EIP_COMMAND_UNREGISTER_SESSION;
        pkt.hdr.session_handle = self.session_handle;
        self.framed.send(pkt).await?;
        Ok(())
    }

    ///  send command: SendRRData
    #[inline]
    pub async fn send_rrdata<'de, D>(&mut self, data: D) -> Result<CommonPacket<'static, E>, E>
    where
        D: Encode,
    {
        let mut buf = BytesMut::new();
        buf.put_u32_le(0); // interface handle, shall be 0 for CIP
        buf.put_u16_le(0); // timeout, 0 for SendUnitData
        buf.put_u16_le(2); //  cpf item count
        buf.put_slice(&[0, 0, 0, 0]); // null address
        buf.put_u16_le(0xB2); // unconnected data item
        buf.put_u16_le(data.bytes_count() as u16);
        data.encode(&mut buf, self.framed.codec_mut())?;

        let mut pkt = EncapsulationPacket {
            hdr: Default::default(),
            data: buf.freeze(),
        };
        pkt.hdr.command = EIP_COMMAND_SEND_RRDATA;
        pkt.hdr.session_handle = self.session_handle;
        let pkt = self.send_and_reply(pkt).await?;
        let interface_handle = LittleEndian::read_u32(&pkt.data[0..4]); // interface handle
        debug_assert_eq!(interface_handle, 0);
        // timeout = &pkt.data[4..6]
        CommonPacket::new(LittleEndianDecoder::<E>::new(pkt.data.slice(6..)))
    }

    /// send command: SendUnitData
    #[inline]
    pub async fn send_unit_data<'de, D>(
        &mut self,
        connection_id: u32,
        sequence_number: u16,
        data: D,
    ) -> Result<CommonPacket<'static, E>, E>
    where
        D: Encode,
    {
        let mut buf = BytesMut::new();
        buf.put_u32_le(0); // interface handle, shall be 0 for CIP
        buf.put_u16_le(0); // timeout, 0 for SendUnitData
        buf.put_u16_le(2); //  cpf item count

        buf.put_u16_le(0xA1); // connected address item
        buf.put_u16_le(4); // data len
        buf.put_u32_le(connection_id);

        buf.put_u16_le(0xB1); // connected data item
        buf.put_u16_le(data.bytes_count() as u16 + 2); // data item len
        buf.put_u16_le(sequence_number);
        data.encode(&mut buf, self.framed.codec_mut())?;

        let mut pkt = EncapsulationPacket {
            hdr: Default::default(),
            data: buf.freeze(),
        };
        pkt.hdr.command = EIP_COMMAND_SEND_UNIT_DATA;
        pkt.hdr.session_handle = self.session_handle;

        let pkt = self.send_and_reply(pkt).await?;
        let interface_handle = LittleEndian::read_u32(&pkt.data[0..4]); // interface handle
        debug_assert_eq!(interface_handle, 0);
        // timeout = &pkt.data[4..6]
        CommonPacketIter::new(LittleEndianDecoder::<E>::new(pkt.data.slice(6..)))
    }
}
