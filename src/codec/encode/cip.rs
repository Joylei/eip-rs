use crate::{
    codec::{ClientCodec, Encodable},
    error::Error,
    frame::cip::{AddressItem, DataItem, MessageRouterRequest, UnconnectedSend},
    objects::socket::SocketAddr,
};
use bytes::{BufMut, Bytes, BytesMut};
use tokio_util::codec::Encoder;

impl From<AddressItem> for Bytes {
    fn from(item: AddressItem) -> Self {
        let mut dst = BytesMut::new();
        match item {
            AddressItem::Null => {
                dst.put_u16(0);
                dst.put_u16(0);
            }
            AddressItem::Connected { connection_id } => {
                dst.put_u16(0xA1);
                dst.put_u16_le(4);
                dst.put_u32_le(connection_id);
            }
            AddressItem::Sequenced {
                connection_id,
                sequence_number,
            } => {
                dst.put_u16(0x8002);
                dst.put_u16_le(8);
                dst.put_u32_le(connection_id);
                dst.put_u32_le(sequence_number);
            }
        }

        dst.freeze()
    }
}

impl From<DataItem> for Bytes {
    fn from(item: DataItem) -> Self {
        let mut dst = BytesMut::new();
        match item {
            DataItem::Unconnected(data) => {
                dst.put_u16(0xB2);
                if let Some(data) = data {
                    dst.reserve(2 + data.len());
                    dst.put_u16_le(data.len() as u16);
                    dst.put_slice(&data);
                } else {
                    dst.put_u16_le(0);
                }
            }
            DataItem::Connected(data) => {
                dst.put_u16(0xB1);
                if let Some(data) = data {
                    dst.reserve(2 + data.len());
                    dst.put_u16_le(data.len() as u16);
                    dst.put_slice(&data);
                } else {
                    dst.put_u16(0);
                }
            }
            DataItem::SockAddr(sock_type, addr) => {
                dst.reserve(20);
                dst.put_u16(sock_type.type_id());
                dst.put_u16_le(16);
                addr.encode(&mut dst).unwrap();
            }
        }
        dst.freeze()
    }
}

impl Encodable for SocketAddr {
    #[inline(always)]
    fn encode(self, dst: &mut bytes::BytesMut) -> Result<(), Error> {
        dst.put_i16(self.sin_family);
        dst.put_u16(self.sin_port);
        dst.put_u32(self.sin_addr);
        dst.put_slice(&self.sin_zero);
        Ok(())
    }

    fn bytes_count(&self) -> usize {
        16
    }
}

// impl SelfEncoder for ForwardCloseRequest {
//     fn encode(self, dst: &mut bytes::BytesMut) -> Result<(), Error> {
//         dst.put_u8(self.time_tick);
//         dst.put_u8(self.timeout_tick);
//         dst.put_u16_le(self.connection_serial_number);
//         dst.put_u16_le(self.originator_vender_id);
//         dst.put_u16_le(self.connection_serial_number);
//         assert!(
//             self.connection_path.len() % 2 == 0 && self.connection_path.len() <= u16::MAX as usize
//         );
//         dst.put_u16_le(self.connection_path.len() as u16 / 2); //path size
//         dst.put_u8(0); // reserved
//         dst.put_slice(&self.connection_path);
//         Ok(())
//     }
// }
