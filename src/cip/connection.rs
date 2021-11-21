// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use bytes::Bytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    /// may be used to reconfigure the connection
    Null = 0,
    /// supported for CIP transport class 0 and class 1
    Multicast = 1,
    /// point to point
    P2P = 2,
}

impl Default for ConnectionType {
    fn default() -> Self {
        Self::Null
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Low = 0,
    High = 1,
    Scheduled = 2,
    Urgent = 3,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Scheduled
    }
}

/// transport direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// act as client
    Client = 0,
    /// act as server
    Server = 1,
}

impl Default for Direction {
    fn default() -> Self {
        Self::Client
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerType {
    Cyclic = 0,
    ChangeOfState = 1,
    /// application object
    Application = 2,
}

impl Default for TriggerType {
    fn default() -> Self {
        Self::Cyclic
    }
}

/// A 16-bit sequence count value is prepended to all Class 1, 2, and 3 transports
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportClass {
    /// either a client OR a server
    Class0 = 0,
    /// either a client OR a server
    Class1 = 1,
    /// will both produce AND consume across this connection
    Class2 = 2,
    /// will both produce AND consume across this connection
    Class3 = 3,
    /// non-blocking
    Class4 = 4,
    /// non-blocking, fragmenting
    Class5 = 5,
    /// multicast, fragmenting
    Class6 = 6,
}

impl Default for TransportClass {
    fn default() -> Self {
        Self::Class0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableLength {
    Fixed = 0,
    Variable = 1,
}

impl Default for VariableLength {
    fn default() -> Self {
        Self::Fixed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadlTimeFormat {
    /// connection is pure data and is modeless
    Modeless = 0,
    /// use zero data length packet to indicate idle mode
    ZeroLength = 1,
    Heartbeat = 3,
    Header32Bit = 4,
}

impl Default for ReadlTimeFormat {
    fn default() -> Self {
        Self::Modeless
    }
}

use crate::cip::epath::{EPath, PortSegment, Segment};
use rand::Rng;

/// forward open connection parameters
#[derive(Debug, Clone)]
pub struct ConnectionParameters {
    /// true (1) indicate more than one owner may be permitted to make a connection
    pub redundant_owner: bool,
    pub connection_type: ConnectionType,
    pub variable_length: VariableLength,
    pub priority: Priority,
    /// The connection size includes the sequence count and the 32-bit real time header, if present
    /// u16 for large open forward;
    /// u8 for open forward, max 505
    pub connection_size: u16,
}

impl Default for ConnectionParameters {
    fn default() -> Self {
        Self {
            redundant_owner: false,
            connection_type: ConnectionType::P2P,
            variable_length: VariableLength::Fixed,
            priority: Priority::High,
            connection_size: 504,
        }
    }
}

///  forward open reply connection parameters
#[derive(Debug)]
pub enum ForwardOpenReply {
    Success {
        service_code: u8,
        reply: ForwardOpenSuccess,
    },
    Fail(ForwardRequestFail),
}

#[derive(Debug, Default)]
pub struct ForwardOpenSuccess {
    /// chosen by target
    pub o_t_connection_id: u32,
    /// from request
    pub t_o_connection_id: u32,
    /// from request
    pub connection_serial_number: u16,
    /// from request
    pub originator_vendor_id: u16,
    /// from request
    pub originator_serial_number: u32,
    /// actual PI, originator to target, microseconds
    pub o_t_api: u32,
    /// actual PI, target to originator, microseconds
    pub t_o_api: u32,
    /// application specific data
    pub app_data: Bytes, // app reply size: u8 | reserved: u8 | reply data
}
#[derive(Debug, Default)]
pub struct ForwardRequestFail {
    /// from request
    pub connection_serial_number: u16,
    /// from request
    pub originator_vendor_id: u16,
    /// from request
    pub originator_serial_number: u32,
    /// size of words ,only present with routing type errors
    pub remaining_path_size: Option<u8>,
}

#[derive(Debug, Default)]
pub struct ForwardCloseRequest<P> {
    pub priority_time_ticks: u8,
    pub timeout_ticks: u8,
    pub connection_serial_number: u16,
    pub originator_vendor_id: u16,
    pub originator_serial_number: u32,
    // padded path
    pub connection_path: P,
}

#[derive(Debug)]
pub enum ForwardCloseReply {
    Success(ForwardCloseSuccess),
    Fail(ForwardRequestFail),
}

#[derive(Debug, Default)]
pub struct ForwardCloseSuccess {
    /// from request
    pub connection_serial_number: u16,
    /// from request
    pub originator_vendor_id: u16,
    /// from request
    pub originator_serial_number: u32,
    /// application specific data
    pub app_data: Bytes, // app reply size: u8 | reserved: u8 | reply data
}

// fn calc_timeout_ticks(timeout: u32) -> (u8, u8) {
//     let time_tick = timeout / 255;
//     let timeout_tick = timeout / (2_u32.pow(time_tick));
//     (time_tick as u8, timeout_tick as u8)
// }

/// CIP connection options, for Forward_Open service request
#[derive(Debug, Clone)]
pub struct Options<P = EPath> {
    pub(crate) o_t_connection_id: u32,
    pub(crate) t_o_connection_id: u32,
    /// tick time in milliseconds
    pub(crate) priority_time_ticks: u8,
    /// tick time in milliseconds
    pub(crate) timeout_ticks: u8,
    pub(crate) connection_serial_number: u16,
    pub(crate) vendor_id: u16,
    pub(crate) originator_serial_number: u32,
    pub(crate) o_t_rpi: u32,
    pub(crate) t_o_rpi: u32,
    /// specifies the multiplier applied to the RPI to obtain the connection timeout value
    pub(crate) timeout_multiplier: u8,
    pub(crate) connection_path: P,
    pub(crate) o_t_params: ConnectionParameters,
    pub(crate) t_o_params: ConnectionParameters,
    pub(crate) transport_direction: Direction,
    pub(crate) transport_class: TransportClass,
    pub(crate) transport_trigger: TriggerType,
    pub(crate) large_open: bool,
}

impl<P> Options<P> {
    #[inline]
    pub fn o_t_connection_id(mut self, val: u32) -> Self {
        self.o_t_connection_id = val;
        self
    }

    #[inline]
    pub fn t_o_connection_id(mut self, val: u32) -> Self {
        self.t_o_connection_id = val;
        self
    }

    #[inline]
    pub fn priority_time_ticks(mut self, val: u8) -> Self {
        self.priority_time_ticks = val & 0xF; // only tick time part, ignore high byte
        self
    }

    #[inline]
    pub fn timeout_ticks(mut self, val: u8) -> Self {
        self.timeout_ticks = val;
        self
    }

    #[inline]
    pub fn connection_serial_number(mut self, val: u16) -> Self {
        self.connection_serial_number = val;
        self
    }

    #[inline]
    pub fn originator_vendor_id(mut self, val: u16) -> Self {
        self.vendor_id = val;
        self
    }

    #[inline]
    pub fn originator_serial_number(mut self, val: u32) -> Self {
        self.originator_serial_number = val;
        self
    }

    #[inline]
    pub fn o_t_rpi(mut self, val: u32) -> Self {
        self.o_t_rpi = val;
        self
    }

    #[inline]
    pub fn t_o_rpi(mut self, val: u32) -> Self {
        self.t_o_rpi = val;
        self
    }

    #[inline]
    pub fn timeout_multiplier(mut self, val: u8) -> Self {
        self.timeout_multiplier = val;
        self
    }

    #[inline]
    pub fn connection_size(mut self, val: u16) -> Self {
        self.o_t_params.connection_size = val;
        self.t_o_params.connection_size = val;
        self
    }

    #[inline]
    pub fn connection_path(mut self, path: P) -> Self {
        self.connection_path = path;
        self
    }

    #[inline]
    pub fn o_t_priority(mut self, val: Priority) -> Self {
        self.o_t_params.priority = val;
        self
    }

    #[inline]
    pub fn t_o_priority(mut self, val: Priority) -> Self {
        self.t_o_params.priority = val;
        self
    }

    #[inline]
    pub fn o_t_variable_length(mut self, val: VariableLength) -> Self {
        self.o_t_params.variable_length = val;
        self
    }

    #[inline]
    pub fn t_o_variable_length(mut self, val: VariableLength) -> Self {
        self.t_o_params.variable_length = val;
        self
    }

    #[inline]
    pub fn t_o_connection_type(mut self, val: ConnectionType) -> Self {
        self.t_o_params.connection_type = val;
        self
    }

    #[inline]
    pub fn o_t_connection_type(mut self, val: ConnectionType) -> Self {
        self.o_t_params.connection_type = val;
        self
    }

    #[inline]
    pub fn o_t_redundant_owner(mut self, val: bool) -> Self {
        self.o_t_params.redundant_owner = val;
        self
    }

    #[inline]
    pub fn t_o_redundant_owner(mut self, val: bool) -> Self {
        self.t_o_params.redundant_owner = val;
        self
    }

    #[inline]
    pub fn transport_direction(mut self, val: Direction) -> Self {
        self.transport_direction = val;
        self
    }

    #[inline]
    pub fn transport_class(mut self, val: TransportClass) -> Self {
        self.transport_class = val;
        self
    }

    #[inline]
    pub fn transport_trigger(mut self, val: TriggerType) -> Self {
        self.transport_trigger = val;
        self
    }

    #[inline]
    pub fn large_open(mut self, val: bool) -> Self {
        self.large_open = val;
        self
    }

    #[inline]
    pub(crate) fn transport_class_trigger(&self) -> u8 {
        let dir = self.transport_direction as u8;
        let trigger = self.transport_trigger as u8;
        let class = self.transport_class as u8;

        (dir << 7) | (trigger << 4) | class
    }
}

impl Default for Options<EPath> {
    #[inline]
    fn default() -> Self {
        // port 1, message router 0x02
        let connection_path = EPath::from(vec![
            Segment::Port(PortSegment::default()),
            Segment::Class(2),
            Segment::Instance(1),
        ]);
        let connection_serial_number: u16 = rand::thread_rng().gen_range(1..0xFFFF);
        Self {
            o_t_connection_id: 0,
            t_o_connection_id: 0,
            priority_time_ticks: 0x03,
            timeout_ticks: 0xfa,
            connection_serial_number,
            vendor_id: 0xFF,
            originator_serial_number: 0xFFFFFFFF,
            o_t_rpi: 0x4240,
            t_o_rpi: 0x4240,
            timeout_multiplier: 3,
            connection_path,
            o_t_params: Default::default(),
            t_o_params: Default::default(),
            transport_direction: Direction::Server,
            transport_class: TransportClass::Class3,
            transport_trigger: TriggerType::Application,
            large_open: false,
        }
    }
}
