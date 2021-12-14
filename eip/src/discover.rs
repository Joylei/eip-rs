// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{
    codec::ClientCodec,
    command::ListIdentity,
    consts::{EIP_COMMAND_LIST_IDENTITY, EIP_DEFAULT_PORT},
};
use bytes::Bytes;
use core::marker::PhantomData;
use futures_util::{stream, SinkExt, Stream, StreamExt};
use rseip_core::{
    cip::{CommonPacketItem, CommonPacketIter},
    codec::{Decode, LittleEndianDecoder},
    Error,
};
use std::{
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};
use tokio::{net::UdpSocket, time};
use tokio_util::udp::UdpFramed;

/// device discovery
#[derive(Debug)]
pub struct EipDiscovery<E> {
    listen_addr: SocketAddrV4,
    broadcast_addr: SocketAddrV4,
    times: Option<usize>,
    interval: Duration,
    _marker: PhantomData<E>,
}

impl<E> EipDiscovery<E> {
    /// create [`EipDiscovery`]
    #[inline]
    pub fn new(listen_addr: Ipv4Addr) -> Self {
        Self {
            listen_addr: SocketAddrV4::new(listen_addr, 0),
            broadcast_addr: SocketAddrV4::new(Ipv4Addr::BROADCAST, EIP_DEFAULT_PORT),
            times: Some(1),
            interval: Duration::from_secs(1),
            _marker: Default::default(),
        }
    }

    /// set broadcast address
    #[inline]
    pub fn broadcast(mut self, ip: Ipv4Addr) -> Self {
        self.broadcast_addr = SocketAddrV4::new(ip, EIP_DEFAULT_PORT);
        self
    }

    /// repeatedly send requests with limited times
    #[inline]
    pub fn repeat(mut self, times: usize) -> Self {
        self.times = Some(times);
        self
    }

    /// repeatedly send requests forever
    #[inline]
    pub fn forever(mut self) -> Self {
        self.times = None;
        self
    }

    /// request interval
    #[inline]
    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }
}

impl<E> EipDiscovery<E>
where
    E: Error + 'static,
{
    /// send requests to discover devices
    pub async fn run<'de, I>(self) -> io::Result<impl Stream<Item = (I, SocketAddr)>>
    where
        I: Decode<'de> + 'static,
    {
        let socket = UdpSocket::bind(self.listen_addr).await?;
        socket.set_broadcast(true)?;
        let service = UdpFramed::new(socket, ClientCodec::<E>::new());
        let (mut tx, rx) = service.split();

        let tx_fut = {
            let broadcast_addr = self.broadcast_addr;
            let interval = self.interval;
            let mut times = self.times;

            async move {
                let rng = std::iter::from_fn(move || match times {
                    Some(0) => None,
                    Some(ref mut v) => {
                        *v -= 1;
                        Some(())
                    }
                    None => Some(()),
                });
                for _ in rng {
                    if tx
                        .send((ListIdentity, broadcast_addr.into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                    time::sleep(interval).await;
                }
            }
        };

        let rx = stream::unfold((rx, Box::pin(tx_fut)), |mut state| async move {
            loop {
                tokio::select! {
                    res = state.0.next() => {
                        if let Some(res) =res {
                            if let Some(v) = res.ok().and_then(|(pkt,addr)| {
                                if pkt.hdr.command != EIP_COMMAND_LIST_IDENTITY {
                                    None
                                } else {
                                    decode_identity::<'_,_,E>(pkt.data).ok().flatten().map(|v| (v, addr))
                                }
                            }) {
                                return Some((v, state))
                            }
                        } else{
                            return None;
                        }
                    },
                    _ = Pin::new(&mut state.1) => {
                        dbg!("cancel rx");
                        return None;
                    },
                }
            }
        });
        Ok(rx)
    }
}

#[inline]
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
