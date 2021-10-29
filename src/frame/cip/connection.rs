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

/// forward open request
pub type ForwardOpenRequest<P> = internal::ForwardOpenRequest<u16, P>;

/// large forward open request
pub type LargeForwardOpenRequest<P> = internal::ForwardOpenRequest<u32, P>;

pub type ForwardOpenConnectionParameters = internal::ConnectionParameters<u16>;

pub type LargeForwardOpenConnectionParameters = internal::ConnectionParameters<u32>;

pub(crate) use internal::ConnectionParameters;

mod internal {
    use super::*;

    /// forward open connection parameters
    #[derive(Debug, Default)]
    pub struct ForwardOpenRequest<S, P> {
        /// connection priority/tick time;
        pub priority_time_ticks: u8,
        /// connection timeout ticks; e.g. 0x0F Timeout is 245 seconds(ticks * tick time)
        pub timeout_ticks: u8,
        /// 0 in request, returned by target
        pub o_t_connection_id: u32,
        /// chosen by originator
        pub t_o_connection_id: u32,
        /// should be unique for the device, chosen by originator
        pub connection_serial_number: u16,
        /// originator vendor id, from Identity Object
        pub vendor_id: u16,
        /// from Identity Object
        pub originator_serial_number: u32, // same as t_o_connection_id?
        /// connection timeout multiplier; multi* RPI inactivity timeout
        pub timeout_multiplier: u8,
        //reserved: [u8; 3],
        /// requested RPI, originator to target, microseconds
        pub o_t_rpi: u32,
        pub o_t_connection_parameters: ConnectionParameters<S>,
        /// Requested RPI, target to originator, microseconds
        pub t_o_rpi: u32,
        pub t_o_connection_parameters: ConnectionParameters<S>,
        pub transport_class_trigger: u8,
        /// connection path
        pub connection_path: P,
    }

    impl<S, P> ForwardOpenRequest<S, P> {
        pub fn with_transport_direction(mut self, dir: Direction) -> Self {
            let dir = dir as u8;
            self.transport_class_trigger = self.transport_class_trigger | (dir << 7);
            self
        }

        pub fn with_transport_trigger(mut self, trigger: TriggerType) -> Self {
            let trigger = trigger as u8;
            self.transport_class_trigger = self.transport_class_trigger | (trigger << 4);
            self
        }

        pub fn with_transport_class(mut self, class: TransportClass) -> Self {
            let class = class as u8;
            self.transport_class_trigger = self.transport_class_trigger | class;
            self
        }

        /// set transport_class_trigger for IO connection
        pub fn with_io_messaging(mut self) -> Self {
            self.transport_class_trigger = 0xF;
            self
        }
    }

    /// forward open connection parameters
    #[derive(Debug, Default)]
    pub struct ConnectionParameters<S> {
        /// true (1) indicate more than one owner may be permitted to make a connection
        pub redundant_owner: bool,
        pub connection_type: ConnectionType,
        pub variable_length: VariableLength,
        pub priority: Priority,
        /// The connection size includes the sequence count and the 32-bit real time header, if present
        /// u16 for large open forward;
        /// u8 for open forward, max 505
        pub connection_size: S,
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
