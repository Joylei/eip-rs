// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

pub mod ab_eip;
pub mod eip;

use crate::{
    adapters::Service,
    cip::{
        connection::{ForwardCloseRequest, ForwardOpenReply, Options},
        epath::EPath,
    },
    service::request::UnconnectedSend,
    Error, Result,
};
use bytes::Bytes;
use futures_util::future::BoxFuture;
use std::{io, sync::atomic::AtomicU16};

pub use ab_eip::{AbEipClient, AbEipConnection, AbEipDriver, AbService};
pub use eip::*;

use crate::{
    cip::{MessageReply, MessageRequest},
    codec::Encodable,
    service::MessageService,
};

/// driver for specified protocol
pub trait Driver {
    /// endpoint, eg: IP address for EIP
    type Endpoint;

    /// driver specific service for CIP
    type Service: Service;

    /// create service
    fn build_service(addr: &Self::Endpoint) -> BoxFuture<Result<Self::Service>>;
}

/// explicit messaging client
#[derive(Debug, Default)]
pub struct Client<B: Driver> {
    /// end point, driver specific
    addr: B::Endpoint,
    /// cip service, driver specific
    service: Option<B::Service>,
    /// connection path
    connection_path: EPath,
}

impl<B: Driver> Client<B> {
    /// create [`Client`]
    #[inline]
    pub fn new(addr: B::Endpoint) -> Self {
        Self {
            addr,
            service: None,
            connection_path: Default::default(),
        }
    }

    /// set connection path
    #[inline]
    pub fn with_connection_path(mut self, path: impl Into<EPath>) -> Self {
        self.connection_path = path.into();
        self
    }

    /// current connection path
    #[inline]
    pub fn connection_path(&self) -> &EPath {
        &self.connection_path
    }

    /// current remote endpoint, driver specific
    #[inline]
    pub fn remote_endpoint(&self) -> &B::Endpoint {
        &self.addr
    }

    #[inline]
    async fn ensure_service(&mut self) -> Result<()> {
        if self.service.is_none() {
            let service = B::build_service(&self.addr).await?;
            self.service = Some(service);
        }
        match self.service {
            None => unreachable!(),
            Some(ref mut service) => {
                service.open().await?;
            }
        }
        Ok(())
    }
}

/// message router request handler
#[async_trait::async_trait(?Send)]
impl<B: Driver> MessageService for Client<B> {
    /// send Heartbeat message to keep underline transport alive
    #[inline]
    async fn heartbeat(&mut self) -> Result<()> {
        if let Some(ref mut service) = self.service {
            let _ = service.heartbeat().await;
        }
        Ok(())
    }

    /// unconnected send
    #[inline]
    async fn send<P, D>(&mut self, mr: MessageRequest<P, D>) -> Result<MessageReply<Bytes>>
    where
        P: Encodable,
        D: Encodable,
    {
        // create service if not created
        self.ensure_service().await?;
        let service = self.service.as_mut().expect("expected service");
        let req = UnconnectedSend::new(self.connection_path.clone(), mr);
        service.unconnected_send(req).await
    }

    /// close underline transport
    #[inline]
    async fn close(&mut self) -> Result<()> {
        if let Some(mut service) = self.service.take() {
            let _ = service.close().await;
        }
        Ok(())
    }

    /// is underline transport closed?
    #[inline]
    fn closed(&self) -> bool {
        self.service.is_none()
    }
}

/// explicit messaging connection
#[derive(Debug)]
pub struct Connection<B: Driver> {
    addr: B::Endpoint,
    origin_options: Options,
    connected_options: Option<Options>,
    /// underline service
    service: Option<B::Service>,
    /// sequence number
    seq_id: AtomicU16,
}

impl<B: Driver> Connection<B> {
    /// Create connection
    #[inline]
    pub fn new(addr: B::Endpoint, options: Options) -> Self {
        Self {
            addr,
            origin_options: options,
            connected_options: None,
            service: None,
            seq_id: Default::default(),
        }
    }

    /// current remote endpoint, driver specific
    #[inline]
    pub fn remote_endpoint(&self) -> &B::Endpoint {
        &self.addr
    }

    /// CIP connection id
    #[inline]
    pub fn connection_id(&self) -> Option<u32> {
        self.connected_options.as_ref().map(|v| v.o_t_connection_id)
    }

    /// is CIP connection built?
    #[inline]
    pub fn connected(&self) -> bool {
        self.connection_id().is_some()
    }

    /// generate next sequence number
    #[inline]
    fn next_sequence_number(&mut self) -> u16 {
        loop {
            let v = self
                .seq_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if v == 0 {
                continue;
            }
            return v;
        }
    }

    /// close current connection and open a new connection
    #[inline]
    pub async fn reconnect(&mut self) -> Result<()> {
        self.close_connection().await?;
        self.open_connection().await?;
        Ok(())
    }

    #[inline]
    async fn ensure_service(&mut self) -> Result<()> {
        if self.service.is_none() {
            let service = B::build_service(&self.addr).await?;
            self.service = Some(service);
        }
        match self.service {
            None => unreachable!(),
            Some(ref mut service) => {
                service.open().await?;
            }
        }
        Ok(())
    }

    /// open connection if not already connected
    #[inline]
    async fn open_connection(&mut self) -> Result<u32> {
        // create service if not created
        self.ensure_service().await?;
        let service = self.service.as_mut().expect("expected service");
        if self.connected_options.is_none() {
            let reply = service.forward_open(self.origin_options.clone()).await?;
            match reply {
                ForwardOpenReply::Success { reply, .. } => {
                    let opts = self
                        .origin_options
                        .clone()
                        .o_t_connection_id(reply.o_t_connection_id)
                        .connection_serial_number(reply.connection_serial_number)
                        .o_t_rpi(reply.o_t_api)
                        .t_o_rpi(reply.t_o_api);
                    self.connected_options = Some(opts);
                }
                ForwardOpenReply::Fail(_) => {
                    return Err(Error::Io(io::Error::new(
                        io::ErrorKind::Other,
                        "ForwardOpen failed",
                    )))
                }
            }
        }
        Ok(self.connection_id().unwrap())
    }

    /// close connection
    #[inline]
    async fn close_connection(&mut self) -> Result<()> {
        if let Some(conn) = self.connected_options.take() {
            if let Some(service) = self.service.as_mut() {
                let request = ForwardCloseRequest {
                    priority_time_ticks: self.origin_options.priority_tick_time,
                    timeout_ticks: self.origin_options.timeout_ticks,
                    connection_serial_number: conn.connection_serial_number,
                    originator_serial_number: conn.originator_serial_number,
                    originator_vendor_id: conn.vendor_id,
                    connection_path: conn.connection_path,
                };
                let _ = service.forward_close(request).await;
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl<B: Driver> MessageService for Connection<B> {
    /// send Heartbeat message to keep underline transport alive
    #[inline]
    async fn heartbeat(&mut self) -> Result<()> {
        if let Some(ref mut service) = self.service {
            let _ = service.heartbeat().await;
            //TODO: is there a way to keep CIP connection alive?
        }
        Ok(())
    }

    /// connected send
    #[inline]
    async fn send<P, D>(&mut self, mr: MessageRequest<P, D>) -> Result<MessageReply<Bytes>>
    where
        P: Encodable,
        D: Encodable,
    {
        // create connection if not connected
        let cid = self.open_connection().await?;
        let sid = self.next_sequence_number();
        let service = self.service.as_mut().expect("expected service");
        service.connected_send(cid, sid, mr).await
    }

    /// close current connection and underline transport
    #[inline]
    async fn close(&mut self) -> Result<()> {
        let _ = self.close_connection().await;
        if let Some(mut service) = self.service.take() {
            let _ = service.close().await;
        }
        Ok(())
    }

    /// is connection closed?
    #[inline]
    fn closed(&self) -> bool {
        self.connected_options.is_none()
    }
}

/// explicit messaging with or without CIP connection
pub enum MaybeConnected<B: Driver> {
    /// unconnected messaging
    Unconnected(Client<B>),
    /// connected messaging
    Connected(Connection<B>),
}

#[async_trait::async_trait(?Send)]
impl<B: Driver> MessageService for MaybeConnected<B> {
    /// send Heartbeat message to keep underline connection/transport alive
    #[inline]
    async fn heartbeat(&mut self) -> Result<()> {
        match self {
            Self::Unconnected(c) => c.heartbeat().await,
            Self::Connected(c) => c.heartbeat().await,
        }
    }

    /// send message request
    #[inline]
    async fn send<P, D>(&mut self, mr: MessageRequest<P, D>) -> Result<MessageReply<Bytes>>
    where
        P: Encodable,
        D: Encodable,
    {
        match self {
            Self::Unconnected(c) => c.send(mr).await,
            Self::Connected(c) => c.send(mr).await,
        }
    }

    /// close underline connection/transport
    #[inline]
    async fn close(&mut self) -> Result<()> {
        match self {
            Self::Unconnected(c) => c.close().await,
            Self::Connected(c) => c.close().await,
        }
    }

    /// underline connection/transport closed?
    #[inline]
    fn closed(&self) -> bool {
        match self {
            Self::Unconnected(c) => c.closed(),
            Self::Connected(c) => c.closed(),
        }
    }
}
