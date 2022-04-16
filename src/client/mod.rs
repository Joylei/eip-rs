// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

/// AB EIP
pub mod ab_eip;
/// generic EIP
pub mod eip;

use crate::{adapters::Service, ClientError, Result};
pub use ab_eip::{AbEipClient, AbEipConnection, AbEipDriver, AbService, AbTemplateService};
use bytes::Bytes;
use core::{
    fmt,
    ops::{Deref, DerefMut},
};
pub use eip::*;
use futures_util::future::BoxFuture;
/// reexport
pub use rseip_cip::connection::OpenOptions;
use rseip_cip::{
    connection::ForwardCloseRequest,
    service::Heartbeat,
    service::{request::UnconnectedSend, MessageService},
    *,
};
use rseip_core::{
    codec::{Decode, Encode},
    Either, Error,
};
use std::{io, sync::atomic::AtomicU16};

/// driver for specified protocol
pub trait Driver: Send + Sync {
    /// endpoint, eg: IP address for EIP
    type Endpoint: fmt::Debug + Clone + Send + Sync;

    /// driver specific service for CIP
    type Service: Service + fmt::Debug + Send + Sync;

    /// create service
    fn build_service(addr: Self::Endpoint) -> BoxFuture<'static, Result<Self::Service>>;
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
            let service = B::build_service(self.addr.clone()).await?;
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

#[async_trait::async_trait]
impl<B: Driver> Heartbeat for Client<B> {
    type Error = ClientError;
    /// send Heartbeat message to keep underline transport alive
    #[inline]
    async fn heartbeat(&mut self) -> Result<()> {
        if let Some(ref mut service) = self.service {
            service.heartbeat().await?;
        }
        Ok(())
    }
}

/// message  request handler
#[async_trait::async_trait]
impl<B: Driver> MessageService for Client<B> {
    type Error = ClientError;

    /// unconnected send
    #[inline]
    async fn send<'de, P, D, R>(&mut self, mr: MessageRequest<P, D>) -> Result<R>
    where
        P: Encode + Send + Sync,
        D: Encode + Send + Sync,
        R: MessageReplyInterface + Decode<'de> + 'static,
    {
        // create service if not created
        self.ensure_service().await?;
        let service = self.service.as_mut().expect("expected service");
        let req = UnconnectedSend::new(self.connection_path.clone(), mr);
        let res = service.unconnected_send(req).await?;
        Ok(res)
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
    origin_options: OpenOptions,
    connected_options: Option<OpenOptions>,
    /// underline service
    service: Option<B::Service>,
    /// sequence number
    seq_id: AtomicU16,
}

impl<B: Driver> Connection<B> {
    /// Create connection
    #[inline]
    pub fn new(addr: B::Endpoint, options: OpenOptions) -> Self {
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
            // NOTE: sequence_number cannot be 0
            if v > 0 {
                return v;
            }
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
            let service = B::build_service(self.addr.clone()).await?;
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
            match reply.into_value() {
                Either::Left(reply) => {
                    let opts = self
                        .origin_options
                        .clone()
                        .o_t_connection_id(reply.o_t_connection_id)
                        .connection_serial_number(reply.connection_serial_number)
                        .o_t_rpi(reply.o_t_api)
                        .t_o_rpi(reply.t_o_api);
                    self.connected_options = Some(opts);
                }
                Either::Right(_) => return Err(Error::custom("forward open failed")),
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

#[async_trait::async_trait]
impl<B: Driver> Heartbeat for Connection<B> {
    type Error = ClientError;

    /// send Heartbeat message to keep underline transport alive
    #[inline]
    async fn heartbeat(&mut self) -> Result<()> {
        if let Some(ref mut service) = self.service {
            service.heartbeat().await?;
            //TODO: is there a way to keep CIP connection alive?
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<B: Driver> MessageService for Connection<B> {
    type Error = ClientError;
    /// connected send
    #[inline]
    async fn send<'de, P, D, R>(&mut self, mr: MessageRequest<P, D>) -> Result<R>
    where
        P: Encode + Send + Sync,
        D: Encode + Send + Sync,
        R: MessageReplyInterface + Decode<'de> + 'static,
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

/// client with CIP connection or without CIP connection
#[derive(Debug)]
pub struct MaybeConnected<B: Driver>(Either<Client<B>, Connection<B>>);

impl<B: Driver> Deref for MaybeConnected<B> {
    type Target = Either<Client<B>, Connection<B>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<B: Driver> DerefMut for MaybeConnected<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait::async_trait]
impl<B: Driver> Heartbeat for MaybeConnected<B> {
    type Error = ClientError;
    /// send Heartbeat message to keep underline connection/transport alive
    #[inline]
    async fn heartbeat(&mut self) -> Result<()> {
        match self.0 {
            Either::Left(ref mut c) => c.heartbeat().await,
            Either::Right(ref mut c) => c.heartbeat().await,
        }
    }
}

#[async_trait::async_trait]
impl<B: Driver> MessageService for MaybeConnected<B> {
    type Error = ClientError;
    /// send message request
    #[inline]
    async fn send<'de, P, D, R>(&mut self, mr: MessageRequest<P, D>) -> Result<R>
    where
        P: Encode + Send + Sync,
        D: Encode + Send + Sync,
        R: MessageReplyInterface + Decode<'de> + 'static,
    {
        match self.0 {
            Either::Left(ref mut c) => c.send(mr).await,
            Either::Right(ref mut c) => c.send(mr).await,
        }
    }

    /// close underline connection/transport
    #[inline]
    async fn close(&mut self) -> Result<()> {
        match self.0 {
            Either::Left(ref mut c) => c.close().await,
            Either::Right(ref mut c) => c.close().await,
        }
    }

    /// underline connection/transport closed?
    #[inline]
    fn closed(&self) -> bool {
        match self.0 {
            Either::Left(ref c) => c.closed(),
            Either::Right(ref c) => c.closed(),
        }
    }
}
