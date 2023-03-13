// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{codec::ClientCodec, consts::EIP_COMMAND_LIST_IDENTITY, EncapsulationPacket};
use asynchronous_codec::{BytesMut, Decoder, Encoder};
use bytes::Bytes;
use core::marker::PhantomData;
use futures_util::{future::poll_fn, ready, stream::FusedStream, Stream};
use pin_project_lite::pin_project;
use rseip_core::{
    cip::{CommonPacketItem, CommonPacketIter},
    codec::{Decode, LittleEndianDecoder},
    Error,
};
use rseip_rt::{AsyncUdpReadHalf, AsyncUdpSocket, AsyncUdpWriteHalf};
use std::{
    io,
    net::{SocketAddr, UdpSocket},
    pin::Pin,
    task::{Context, Poll},
};

pin_project! {
    pub struct Discovery<S, I, E> {
        codec: ClientCodec<E>,
        broadcast: SocketAddr,
        #[pin]
        socket: S,
        reply_buf: BytesMut,
        send_buf: Option<Bytes>,
        _maker: PhantomData<I>,
    }
}

impl<S, I, E> Discovery<S, I, E>
where
    S: AsyncUdpSocket + Unpin,
    E: Error + 'static,
{
    pub fn new(listen_addr: SocketAddr, broadcast_addr: SocketAddr) -> io::Result<Self> {
        let socket = UdpSocket::bind(listen_addr)?;
        socket.set_broadcast(true)?;
        socket.set_nonblocking(true)?;
        let socket = S::from_std(socket)?;
        Ok(Self {
            codec: ClientCodec::new(),
            broadcast: broadcast_addr,
            socket,
            reply_buf: BytesMut::with_capacity(4096),
            send_buf: None,
            _maker: Default::default(),
        })
    }

    pub fn split(self) -> (Sender<S, E>, Receiver<S, I, E>) {
        let (r, w) = self.socket.split();
        let sender = Sender {
            codec: self.codec,
            socket: w,
            send_buf: self.send_buf,
            broadcast: self.broadcast,
        };
        let receiver = Receiver {
            codec: ClientCodec::new(),
            socket: r,
            reply_buf: self.reply_buf,
            _marker: Default::default(),
        };
        (sender, receiver)
    }

    pub async fn send(&mut self) -> Result<(), E> {
        if self.send_buf.is_none() {
            let mut pkt = EncapsulationPacket::default();
            pkt.hdr.command = EIP_COMMAND_LIST_IDENTITY;
            let mut buf = BytesMut::new();
            self.codec.encode(pkt, &mut buf)?;
            self.send_buf = Some(buf.freeze());
        }
        let buf = self.send_buf.as_ref().unwrap();
        poll_fn(|cx| self.socket.poll_write(cx, &buf, self.broadcast)).await?;
        Ok(())
    }

    pub fn poll_reply<'de>(self: Pin<&mut Self>, cx: &mut Context) -> Poll<(SocketAddr, I)>
    where
        I: Decode<'de> + 'static,
    {
        let it = self.get_mut();
        it.reply_buf.clear();
        if let Ok((_len, addr)) = ready!(it.socket.poll_read(cx, &mut it.reply_buf)) {
            if let Ok(Some(pkt)) = it.codec.decode(&mut it.reply_buf) {
                if pkt.hdr.command == EIP_COMMAND_LIST_IDENTITY {
                    if let Some(res) = decode_identity::<I, E>(pkt.data).ok().flatten() {
                        return Poll::Ready((addr, res));
                    }
                }
            }
        }
        Poll::Pending
    }
}

impl<'de, S, I, E> Stream for Discovery<S, I, E>
where
    S: AsyncUdpSocket,
    E: Error + 'static,
    I: Decode<'de> + 'static,
{
    type Item = (SocketAddr, I);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let reply = ready!(self.poll_reply(cx));
        Poll::Ready(Some(reply))
    }
}
impl<'de, S, I, E> FusedStream for Discovery<S, I, E>
where
    S: AsyncUdpSocket,
    E: Error + 'static,
    I: Decode<'de> + 'static,
{
    fn is_terminated(&self) -> bool {
        false
    }
}

fn decode_identity<'de, I, E>(data: Bytes) -> Result<Option<I>, E>
where
    I: Decode<'de> + 'static,
    E: Error + 'static,
{
    let mut cpf = CommonPacketIter::new(LittleEndianDecoder::<E>::new(data))?;
    if let Some(item) = cpf.next_typed() {
        let item: CommonPacketItem<I> = item?;
        item.ensure_type_code::<E>(0x0C)?;
        return Ok(Some(item.data));
    }
    Ok(None)
}

pin_project! {
    #[derive(Debug)]
    pub struct Sender<S, E> {
        codec: ClientCodec<E>,
        socket: AsyncUdpWriteHalf<S>,
        send_buf: Option<Bytes>,
        broadcast: SocketAddr,
    }
}

impl<S, E> Sender<S, E>
where
    S: AsyncUdpSocket + Unpin,
    E: Error + 'static,
{
    pub async fn send(&mut self) -> Result<(), E> {
        if self.send_buf.is_none() {
            let mut pkt = EncapsulationPacket::default();
            pkt.hdr.command = EIP_COMMAND_LIST_IDENTITY;
            let mut buf = BytesMut::new();
            self.codec.encode(pkt, &mut buf)?;
            self.send_buf = Some(buf.freeze());
        }
        let buf = self.send_buf.as_ref().unwrap();
        poll_fn(|cx| self.socket.poll_write(cx, &buf, self.broadcast)).await?;
        Ok(())
    }
}

pin_project! {
    #[derive(Debug)]
    pub struct Receiver<S, I, E> {
        codec: ClientCodec<E>,
        #[pin]
        socket: AsyncUdpReadHalf<S>,
        reply_buf: BytesMut,
        _marker: PhantomData<I>,
    }
}

impl<'de, S, I, E> Stream for Receiver<S, I, E>
where
    S: AsyncUdpSocket + Unpin,
    E: Error + 'static,
    I: Decode<'de> + 'static,
{
    type Item = (SocketAddr, I);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let it = self.get_mut();
        it.reply_buf.clear();
        if let Ok((_len, addr)) = ready!(it.socket.poll_read(cx, &mut it.reply_buf)) {
            if let Ok(Some(pkt)) = it.codec.decode(&mut it.reply_buf) {
                dbg!(&pkt);
                if pkt.hdr.command == EIP_COMMAND_LIST_IDENTITY {
                    if let Some(res) = decode_identity::<I, E>(pkt.data).ok().flatten() {
                        return Poll::Ready(Some((addr, res)));
                    }
                }
            }
        }
        Poll::Pending
    }
}
