// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::{
    command::{self, Command},
    framed::Framed,
    CommonPacketItem, EncapsulationPacket,
};
use crate::{codec::ClientCodec, *};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, Bytes, BytesMut};
use core::fmt;
use futures_util::{SinkExt, StreamExt};
use rseip_core::{cip::CommonPacketIterator, InnerError};
use std::{convert::TryFrom, io};
use tokio::io::{AsyncRead, AsyncWrite};

/// EIP context
pub struct EipContext<T> {
    framed: Framed<T, ClientCodec>,
    session_handle: u32,
    #[allow(unused)]
    sender_context: Bytes,
}

impl<T> fmt::Debug for EipContext<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EipContext")
            .field("session_handle", &self.session_handle)
            .field("sender_context", &self.sender_context)
            .field("framed", &"<Framed>")
            .finish()
    }
}

impl<T> EipContext<T> {
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

impl<T> EipContext<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    /// create [`EipContext`]
    #[inline]
    pub fn new(transport: T) -> Self {
        let framed = Framed::new(transport, ClientCodec {});

        Self {
            framed,
            session_handle: 0,
            sender_context: Bytes::from_static(&[0, 0, 0, 0, 0, 0, 0, 0]),
        }
    }

    /// send and wait for reply
    #[inline]
    async fn send_and_reply<C, F, R, E>(&mut self, cmd: C, f: F) -> StdResult<R, E>
    where
        C: Command,
        F: FnOnce(EncapsulationPacket<Bytes>) -> StdResult<R, E>,
        E: From<crate::Error>,
    {
        let code = C::command_code();
        log::trace!("send command: {}", code);
        self.framed.send(cmd).await?;
        match self.framed.next().await {
            Some(item) => {
                let pkt = item?;
                pkt.hdr.ensure_command(code).map_err(|e| Error::from(e))?;
                let res = f(pkt)?;
                Ok(res)
            }
            None => Err(Error::from(io::ErrorKind::ConnectionAborted).into()),
        }
    }

    /// send command: NOP
    #[inline]
    pub async fn nop<F, E>(&mut self, data: Frame<F, E>) -> Result<()>
    where
        F: FnOnce(&mut BytesMut) -> StdResult<(), E>,
    {
        log::trace!("send command: NOP");
        self.framed.send(command::Nop { data }).await?;
        Ok(())
    }

    /// send command: ListIdentity
    #[allow(unused)]
    #[inline]
    pub async fn list_identity<R>(&mut self) -> Result<CommonPacketIterator>
    where
        R: TryFrom<CommonPacketItem>,
        R::Error: From<io::Error>,
    {
        let res = self
            .send_and_reply::<_, _, _, Error>(command::ListIdentity, |pkt| {
                let cpf = CommonPacketIterator::new(pkt.data)?;
                Ok(cpf)
            })
            .await?;
        Ok(res)
    }

    /// send command: ListServices
    #[allow(unused)]
    #[inline]
    pub async fn list_service<R>(&mut self) -> Result<CommonPacketIterator>
    where
        R: TryFrom<CommonPacketItem>,
        R::Error: From<io::Error>,
    {
        let res = self
            .send_and_reply::<_, _, _, Error>(command::ListServices, |pkt| {
                let cpf = CommonPacketIterator::new(pkt.data)?;
                Ok(cpf)
            })
            .await?;
        Ok(res)
    }

    /// send command: ListInterface
    #[allow(unused)]
    #[inline]
    pub async fn list_interface<R>(&mut self) -> Result<CommonPacketIterator>
    where
        R: TryFrom<CommonPacketItem>,
        R::Error: From<io::Error>,
    {
        let res = self
            .send_and_reply::<_, _, _, Error>(command::ListInterfaces, |pkt| {
                let cpf = CommonPacketIterator::new(pkt.data)?;
                Ok(cpf)
            })
            .await?;
        Ok(res)
    }

    /// send command: RegisterSession
    #[inline]
    pub async fn register_session(&mut self) -> Result<u32> {
        let session_handle = self
            .send_and_reply::<_, _, _, Error>(command::RegisterSession, |pkt| {
                let session_handle = pkt.hdr.session_handle;
                let reply_data = pkt.data;
                if reply_data.len() != 4 {
                    return Err(Error::from(InnerError::InvalidData)
                        .with_context("ENIP command reply: invalid data"));
                }
                #[cfg(debug_assertions)]
                {
                    let protocol_version = LittleEndian::read_u16(&reply_data[0..2]);
                    debug_assert_eq!(protocol_version, 1);
                    let session_options = LittleEndian::read_u16(&reply_data[2..4]);
                    debug_assert_eq!(session_options, 0);
                }
                if session_handle == 0 {
                    return Err(Error::from(InnerError::InvalidData)
                        .with_context("ENIP command reply: invalid session handle"));
                }
                Ok(session_handle)
            })
            .await?;
        self.session_handle = session_handle;
        Ok(session_handle)
    }

    /// send command: UnRegisterSession
    #[inline]
    pub async fn unregister_session(&mut self) -> Result<()> {
        if self.session_handle == 0 {
            return Ok(());
        }
        log::trace!("send command: UnRegisterSession");
        self.framed
            .send(command::UnRegisterSession {
                session_handle: self.session_handle,
            })
            .await?;
        Ok(())
    }

    ///  send command: SendRRData
    #[inline]
    pub async fn send_rrdata<F, E>(
        &mut self,
        data: Frame<F, E>,
    ) -> StdResult<CommonPacketIterator, E>
    where
        F: FnOnce(&mut BytesMut) -> StdResult<(), E>,
        E: From<Error> + From<io::Error>,
    {
        let res = self
            .send_and_reply::<_, _, _, E>(
                command::SendRRData {
                    session_handle: self.session_handle,
                    timeout: 0,
                    data,
                },
                |pkt| {
                    let interface_handle = LittleEndian::read_u32(&pkt.data[0..4]); // interface handle
                    debug_assert_eq!(interface_handle, 0);
                    // timeout = &pkt.data[4..6]
                    CommonPacketIterator::new(pkt.data.slice(6..)).map_err(|e| e.into())
                },
            )
            .await?;
        Ok(res)
    }

    /// send command: SendUnitData
    #[inline]
    pub async fn send_unit_data<F, E>(
        &mut self,
        connection_id: u32,
        sequence_number: u16,
        data: Frame<F, E>,
    ) -> StdResult<CommonPacketIterator, E>
    where
        F: FnOnce(&mut BytesMut) -> StdResult<(), E>,
        E: From<Error> + From<io::Error>,
    {
        let res = self
            .send_and_reply::<_, _, _, E>(
                command::SendUnitData {
                    session_handle: self.session_handle,
                    sequence_number,
                    connection_id,
                    data,
                },
                |pkt| {
                    let interface_handle = LittleEndian::read_u32(&pkt.data[0..4]); // interface handle
                    debug_assert_eq!(interface_handle, 0);
                    // timeout = &pkt.data[4..6]
                    CommonPacketIterator::new(pkt.data.slice(6..)).map_err(|e| e.into())
                },
            )
            .await?;
        Ok(res)
    }
}
