use anyhow::Result;
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, BytesMut};
use rseip::{
    cip::{
        connection::Options,
        epath::{EPath, Segment},
        MessageRouterRequest,
    },
    client::EipConnection,
    codec::Encodable,
    consts::EIP_DEFAULT_PORT,
    service::MessageRouter,
};
use std::net::SocketAddrV4;

#[tokio::main]
pub async fn main() -> Result<()> {
    let endpoint = SocketAddrV4::new("192.168.0.83".parse()?, EIP_DEFAULT_PORT);
    let mut connection = EipConnection::new(endpoint, Options::default());
    let mr_request = MessageRouterRequest::new(
        0x4C,
        EPath::from(vec![Segment::Symbol("test_car1_x".to_owned())]),
        ElementCount(1),
    );

    let resp = connection.send(mr_request).await?;
    assert_eq!(resp.reply_service, 0xCC); // tag read service
    assert_eq!(LittleEndian::read_u16(&resp.data[0..2]), 0xC4); // DINT

    connection.close().await?;
    Ok(())
}

struct ElementCount(u16);

impl Encodable for ElementCount {
    fn encode(self, dst: &mut BytesMut) -> rseip::Result<()> {
        dst.put_u16_le(self.0);
        Ok(())
    }
    fn bytes_count(&self) -> usize {
        2
    }
}
