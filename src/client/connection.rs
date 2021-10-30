// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2020-2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::Client;
use crate::{
    codec::Encodable,
    error::Error,
    frame::cip::{
        connection::*, EPath, MessageRouterReply, MessageRouterRequest, PortSegment, Segment,
    },
    service::client::TcpService,
    Result,
};
use bytes::Bytes;
use rand::Rng;
use std::io;
use std::net::SocketAddr;

fn calc_timeout_ticks(timeout: u32) -> (u8, u8) {
    let time_tick = timeout / 255;
    let timeout_tick = timeout / (2_u32.pow(time_tick));
    (time_tick as u8, timeout_tick as u8)
}

#[derive(Debug)]
pub struct Options {
    /// tick time in milliseconds
    priority_time_ticks: u8,
    /// tick time in milliseconds
    timeout_ticks: u8,
    connection_serial_number: u16,
    vendor_id: u16,
    originator_serial_number: u32,
    o_t_rpi: u32,
    t_o_rpi: u32,
    /// specifies the multiplier applied to the RPI to obtain the connection timeout value
    timeout_multiplier: u8,
    connection_size: u16,
    connection_path: EPath,
    o_t_priority: Priority,
    t_o_priority: Priority,
    o_t_variable_length: VariableLength,
    t_o_variable_length: VariableLength,
    o_t_connection_type: ConnectionType,
    t_o_connection_type: ConnectionType,
    transport_direction: Direction,
    transport_class: TransportClass,
    transport_trigger: TriggerType,
    large_open: bool,
}

impl Options {
    #[inline(always)]
    pub fn priority_time_ticks(mut self, val: u8) -> Self {
        self.priority_time_ticks = val & 0xF; // only tick time part, ignore high byte
        self
    }
    #[inline(always)]
    pub fn timeout_ticks(mut self, val: u8) -> Self {
        self.timeout_ticks = val;
        self
    }
    #[inline(always)]
    pub fn connection_serial_number(mut self, val: u16) -> Self {
        self.connection_serial_number = val;
        self
    }
    #[inline(always)]
    pub fn originator_vendor_id(mut self, val: u16) -> Self {
        self.vendor_id = val;
        self
    }
    #[inline(always)]
    pub fn originator_serial_number(mut self, val: u32) -> Self {
        self.originator_serial_number = val;
        self
    }
    #[inline(always)]
    pub fn o_t_rpi(mut self, val: u32) -> Self {
        self.o_t_rpi = val;
        self
    }
    #[inline(always)]
    pub fn t_o_rpi(mut self, val: u32) -> Self {
        self.t_o_rpi = val;
        self
    }
    #[inline(always)]
    pub fn timeout_multiplier(mut self, val: u8) -> Self {
        self.timeout_multiplier = val;
        self
    }
    #[inline(always)]
    pub fn connection_size(mut self, val: u16) -> Self {
        self.connection_size = val;
        self
    }
    #[inline(always)]
    pub fn connection_path(mut self, path: EPath) -> Self {
        self.connection_path = path;
        self
    }
    #[inline(always)]
    pub fn o_t_priority(mut self, val: Priority) -> Self {
        self.o_t_priority = val;
        self
    }
    #[inline(always)]
    pub fn t_o_priority(mut self, val: Priority) -> Self {
        self.t_o_priority = val;
        self
    }
    #[inline(always)]
    pub fn o_t_variable_length(mut self, val: VariableLength) -> Self {
        self.o_t_variable_length = val;
        self
    }
    #[inline(always)]
    pub fn t_o_variable_length(mut self, val: VariableLength) -> Self {
        self.t_o_variable_length = val;
        self
    }
    #[inline(always)]
    pub(crate) fn transport_direction(mut self, val: Direction) -> Self {
        self.transport_direction = val;
        self
    }
    #[inline(always)]
    pub(crate) fn transport_class(mut self, val: TransportClass) -> Self {
        self.transport_class = val;
        self
    }
    #[inline(always)]
    pub(crate) fn transport_trigger(mut self, val: TriggerType) -> Self {
        self.transport_trigger = val;
        self
    }
    #[inline(always)]
    pub fn large_open(mut self, val: bool) -> Self {
        self.large_open = val;
        self
    }
    #[inline(always)]
    pub async fn open(self, addr: SocketAddr) -> Result<Connection> {
        let client = Client::internal_connect(addr).await?;
        let res = Connection::new(client, self).await;
        match res {
            Ok(c) => Ok(c),
            Err((mut client, e)) => {
                let _ = client.close().await;
                Err(e)
            }
        }
    }
}

impl Default for Options {
    #[inline]
    fn default() -> Self {
        let connection_serial_number: u16 = rand::thread_rng().gen_range(1..0xFFFF);
        Self {
            priority_time_ticks: 0x03,
            timeout_ticks: 0xfa,
            connection_serial_number,
            vendor_id: 0xFF,
            originator_serial_number: 0xFFFFFFFF,
            o_t_rpi: 0x4240,
            t_o_rpi: 0x4240,
            timeout_multiplier: 3,
            connection_size: 504,
            connection_path: EPath::from(vec![
                Segment::Port(PortSegment::default()),
                Segment::Class(2),
                Segment::Instance(1),
            ]),
            o_t_priority: Priority::High,
            t_o_priority: Priority::High,
            o_t_variable_length: VariableLength::Fixed,
            t_o_variable_length: VariableLength::Fixed,
            o_t_connection_type: ConnectionType::P2P,
            t_o_connection_type: ConnectionType::P2P,
            transport_direction: Direction::Server,
            transport_class: TransportClass::Class3,
            transport_trigger: TriggerType::Application,
            large_open: false,
        }
    }
}

/// CIP connection
pub struct Connection {
    client: Client,
    sequence_number: u16,
    connection_serial_number: u16,
    originator_serial_number: u32,
    originator_vendor_id: u16,
    connection_path: EPath,
    o_t_connection_id: Option<u32>,
}

impl Connection {
    /// create options to build [`Connection`]
    #[inline(always)]
    pub fn options() -> Options {
        Options::default()
    }

    /// create [`Connection`]
    pub async fn new(
        mut client: Client,
        options: Options,
    ) -> std::result::Result<Self, (Client, Error)> {
        let service = &mut client.0;

        let res = if options.large_open {
            let request = LargeForwardOpenRequest {
                priority_time_ticks: options.priority_time_ticks,
                timeout_ticks: options.timeout_ticks,
                o_t_connection_id: 0,
                t_o_connection_id: 0,
                connection_serial_number: options.connection_serial_number,
                vendor_id: options.vendor_id,
                originator_serial_number: options.originator_serial_number,
                timeout_multiplier: options.timeout_multiplier,
                o_t_rpi: options.o_t_rpi,
                t_o_rpi: options.t_o_rpi,
                o_t_connection_parameters: LargeForwardOpenConnectionParameters {
                    redundant_owner: false,
                    connection_type: options.o_t_connection_type,
                    connection_size: options.connection_size as u32,
                    variable_length: options.o_t_variable_length,
                    priority: options.o_t_priority,
                },
                t_o_connection_parameters: LargeForwardOpenConnectionParameters {
                    redundant_owner: false,
                    connection_type: options.t_o_connection_type,
                    connection_size: options.connection_size as u32,
                    variable_length: options.t_o_variable_length,
                    priority: options.t_o_priority,
                },
                connection_path: options.connection_path.clone(),
                ..Default::default()
            }
            .with_transport_direction(options.transport_direction)
            .with_transport_class(options.transport_class)
            .with_transport_trigger(options.transport_trigger);
            service.large_forward_open(request).await
        } else {
            let request = ForwardOpenRequest {
                priority_time_ticks: options.priority_time_ticks,
                timeout_ticks: options.timeout_ticks,
                o_t_connection_id: 0,
                t_o_connection_id: 0,
                connection_serial_number: options.connection_serial_number,
                vendor_id: options.vendor_id,
                originator_serial_number: options.originator_serial_number,
                timeout_multiplier: options.timeout_multiplier,
                o_t_rpi: options.o_t_rpi,
                t_o_rpi: options.t_o_rpi,
                o_t_connection_parameters: ForwardOpenConnectionParameters {
                    redundant_owner: false,
                    connection_type: options.o_t_connection_type,
                    connection_size: options.connection_size,
                    variable_length: options.o_t_variable_length,
                    priority: options.o_t_priority,
                },
                t_o_connection_parameters: ForwardOpenConnectionParameters {
                    redundant_owner: false,
                    connection_type: options.t_o_connection_type,
                    connection_size: options.connection_size,
                    variable_length: options.t_o_variable_length,
                    priority: options.t_o_priority,
                },
                connection_path: options.connection_path.clone(),
                ..Default::default()
            }
            .with_transport_direction(options.transport_direction)
            .with_transport_class(options.transport_class)
            .with_transport_trigger(options.transport_trigger);
            service.forward_open(request).await
        };

        match res {
            Err(e) => Err((client, e)),
            Ok(ForwardOpenReply::Fail(_)) => Err((
                client,
                io::Error::new(io::ErrorKind::Other, "forward open: failed").into(),
            )),
            Ok(ForwardOpenReply::Success { reply, .. }) => Ok(Connection {
                client,
                sequence_number: 0,
                connection_serial_number: reply.connection_serial_number,
                originator_vendor_id: options.vendor_id,
                originator_serial_number: options.originator_serial_number,
                o_t_connection_id: reply.o_t_connection_id.into(),
                connection_path: options.connection_path,
            }),
        }
    }

    #[inline(always)]
    pub fn inner_mut(&mut self) -> &mut Client {
        &mut self.client
    }

    /// connected send
    #[inline]
    pub async fn send<P, D>(
        &mut self,
        mr: MessageRouterRequest<P, D>,
    ) -> Result<MessageRouterReply<Bytes>>
    where
        P: Encodable,
        D: Encodable,
    {
        let connection_id = match self.o_t_connection_id {
            Some(id) => id,
            None => {
                return Err(
                    io::Error::new(io::ErrorKind::Other, "connection has been closed").into(),
                )
            }
        };

        //generate new sequence number
        self.sequence_number = if self.sequence_number == u16::MAX {
            1
        } else {
            self.sequence_number + 1
        };

        let context = &mut self.client.0;

        let reply = context
            .connected_send(connection_id, self.sequence_number, mr)
            .await?;
        Ok(reply)
    }

    /// close connection
    #[inline]
    pub async fn close(&mut self) -> Result<()> {
        if self.o_t_connection_id.is_some() {
            let context = &mut self.client.0;
            let request = ForwardCloseRequest {
                priority_time_ticks: 0x03,
                timeout_ticks: 0xfa,
                connection_serial_number: self.connection_serial_number,
                originator_serial_number: self.originator_serial_number,
                originator_vendor_id: self.originator_vendor_id,
                connection_path: self.connection_path.clone(),
            };
            let reply = context.forward_close(request).await?;
            match reply {
                ForwardCloseReply::Fail(_) => {
                    return Err(
                        io::Error::new(io::ErrorKind::Other, "forward close: failed").into(),
                    );
                }
                ForwardCloseReply::Success { .. } => self.o_t_connection_id = None,
            }
        }
        Ok(())
    }

    /// is CIP connection closed
    #[inline(always)]
    pub fn is_closed(&self) -> bool {
        self.o_t_connection_id.is_none()
    }

    #[inline(always)]
    pub fn into_inner(self) -> Client {
        self.client
    }
}
