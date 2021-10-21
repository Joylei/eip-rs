use super::socket::SocketAddr;

/// class code = 0x01
/// type code =  0x0C
/// type_code: u16 | item_len: u16 | item_data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityObject {
    /// encapsulation protocol version supported
    pub protocol_version: u16,
    pub socket_addr: SocketAddr,
    /// device manufacturers vendor id
    pub vendor_id: u16,
    /// device type of product
    pub device_type: u16,
    /// product code
    pub product_code: u16,
    /// device revision
    pub revision: [u8; 2],
    /// current status of device
    pub status: u16,
    /// serial number of device
    pub serial_number: u32,
    pub product_name_len: u8,
    /// short string
    pub product_name: String,
    /// current state of device
    pub state: u8,
}
