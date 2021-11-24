// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{
    codec::ClientCodec,
    command::ListIdentity,
    consts::{EIP_COMMAND_LIST_IDENTITY, EIP_DEFAULT_PORT},
    CommonPacket, StdResult,
};
use bytes::Bytes;
use futures_util::{stream, SinkExt, Stream, StreamExt};
use std::{
    convert::TryFrom,
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};
use tokio::{net::UdpSocket, sync::Notify, time};
use tokio_util::udp::UdpFramed;

/// device discovery
#[derive(Debug)]
pub struct EipDiscovery {
    listen_addr: SocketAddrV4,
    broadcast_addr: SocketAddrV4,
    times: Option<usize>,
    interval: Duration,
}

impl EipDiscovery {
    /// create [`EipDiscovery`]
    #[inline]
    pub fn new(listen_addr: Ipv4Addr) -> Self {
        Self {
            listen_addr: SocketAddrV4::new(listen_addr, 0),
            broadcast_addr: SocketAddrV4::new(Ipv4Addr::BROADCAST, EIP_DEFAULT_PORT),
            times: Some(1),
            interval: Duration::from_secs(1),
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

impl EipDiscovery {
    /// send requests to discover devices
    pub async fn run<I, E>(self) -> io::Result<impl Stream<Item = (I, SocketAddr)>>
    where
        I: TryFrom<Bytes>,
        I::Error: Into<E>,
        E: From<io::Error>,
    {
        let socket = UdpSocket::bind(self.listen_addr).await?;
        socket.set_broadcast(true)?;
        let service = UdpFramed::new(socket, ClientCodec {});
        let (mut tx, rx) = service.split();
        let notify = Arc::new(Notify::new());
        {
            let broadcast_addr = self.broadcast_addr;
            let interval = self.interval;
            let mut times = self.times;
            let notify = notify.clone();

            tokio::spawn(async move {
                let fut = async {
                    let rng = std::iter::from_fn(move || match times {
                        Some(0) => None,
                        Some(ref mut v) => {
                            *v -= 1;
                            Some(())
                        }
                        None => Some(()),
                    });
                    for _ in rng {
                        if let Err(_) = tx.send((ListIdentity, broadcast_addr.into())).await {
                            break;
                        }
                        time::sleep(interval).await;
                    }
                };
                tokio::select! {
                    _ = fut => {
                        notify.notify_waiters();
                    },
                    _ = notify.notified() => {
                        dbg!("cancel tx");
                    }
                }
            });
        }

        let rx = stream::unfold(State { inner: rx, notify }, |mut state| async move {
            loop {
                tokio::select! {
                    res = state.inner.next() => {
                        let res = match res {
                            Some(v) => v,
                            None => {
                                state.notify.notify_waiters();
                                return None;
                            }
                        };
                        let res = res.ok().and_then(|(pkt,addr)| {
                            if pkt.hdr.command != EIP_COMMAND_LIST_IDENTITY {
                                None
                            } else {
                                decode_identity(pkt.data).ok().flatten().map(|v| (v, addr))
                            }
                        });
                        match res {
                            Some(v) => return Some((v, state)),
                            None => continue,
                        }
                    },
                    _ = state.notify.notified() => {
                        dbg!("cancel rx");
                        return None;
                    },
                }
            }
        });
        Ok(rx)
    }
}

struct State<S> {
    inner: S,
    /// notify cancellation
    notify: Arc<Notify>,
}

impl<S> Drop for State<S> {
    #[inline]
    fn drop(&mut self) {
        self.notify.notify_waiters();
    }
}

#[inline]
fn decode_identity<I, E>(data: Bytes) -> StdResult<Option<I>, E>
where
    I: TryFrom<Bytes>,
    I::Error: Into<E>,
    E: From<io::Error>,
{
    let cpf = CommonPacket::try_from(data)?;
    for item in cpf.into_iter() {
        item.ensure_type_code(0x0C)?;
        let res = I::try_from(item.data).map_err(|e| e.into())?;
        return Ok(Some(res));
    }
    Ok(None)
}
