use bytes::Bytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    Null,
    /// supported for CIP transport class 0 and class 1
    Multicast,
    /// point to point
    P2P,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Low,
    High,
    /// for future use
    Scheduled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerType {
    Cyclic,
    ChangeOfState,
    Application,
}

/// connection parameters
pub struct Parameters {
    /// 1-second ticks; e.g. 0x0A
    connection_priority_tick: u8,
    /// ticks * tick time
    connection_timeout_ticks: u8,
    /// 0 in request, returned by target
    o_t_connection_id: u32,
    /// chosen by originator
    t_o_connection_id: u32,
    /// chosen by originator
    connection_serial_number: u16,
    /// from Identity Object
    vender_id: u16,
    /// from Identity Object
    originator_serial_number: u16, // same as t_o_connection_id
    /// multi* RPI inactivity timeout
    connection_timeout_multiplier: u8,
    //reserved: [u8; 3],
    /// requested rpi, originator to target, microseconds
    o_t_rpi: u32,
    o_t_connection_parameters: u16,
    t_o_rpi: u32,
    t_o_connection_parameters: u16,
    transport_class_trigger: u8,

    connection_type: ConnectionType,
    priority: Priority,
    trigger_type: TriggerType,
    /// no larger than 65511, Forward_Open limits to 511
    connection_size: u16,
    /// connection request timeout
    connection_timeout: u32,
    connection_path: Bytes,
}

pub struct ForwardCloseRequest {
    pub time_tick: u8,
    pub timeout_tick: u8,
    pub connection_serial_number: u16,
    pub originator_vender_id: u16,
    pub originator_serial_number: u32,
    // padded path
    pub connection_path: Bytes,
}
