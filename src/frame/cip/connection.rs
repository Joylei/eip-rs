use bytes::Bytes;

use crate::codec::Encodable;

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

/// forward open connection parameters
#[derive(Debug, Default)]
pub struct ConnectionParameters<P> {
    /// connection priority/tick time; 1-second ticks; e.g. 0x0A
    pub priority_time_ticks: u8,
    /// connection timeout ticks; e.g. 0x0F Timeout is 245 seconds(ticks * tick time)
    pub timeout_ticks: u8,
    /// 0 in request, returned by target
    pub o_t_connection_id: u32,
    /// chosen by originator
    pub t_o_connection_id: u32,
    /// chosen by originator
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
    pub o_t_connection_parameters: u16,
    /// Requested RPI, target to originator, microseconds
    pub t_o_rpi: u32,
    pub t_o_connection_parameters: u16,
    /// 0xA3, Server transport, class3, application trigger
    pub transport_class_trigger: u8,
    /// connection path
    pub connection_path: P,
}

///  forward open reply connection parameters
#[derive(Debug, Default)]
pub struct ForwardOpenReply {
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
    pub app_data: Bytes,
}

#[derive(Debug, Default)]
pub struct ForwardCloseRequest<P> {
    pub priority_time_ticks: u8,
    pub timeout_tick: u8,
    pub connection_serial_number: u16,
    pub originator_vender_id: u16,
    pub originator_serial_number: u32,
    // padded path
    pub connection_path: P,
}
