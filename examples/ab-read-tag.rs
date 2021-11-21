use anyhow::Result;
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, BytesMut};
use rseip::{
    cip::{
        epath::{EPath, PortSegment, Segment},
        MessageRouterRequest,
    },
    client::EipClient,
    codec::Encodable,
    consts::EIP_DEFAULT_PORT,
    service::MessageRouter,
};
use std::net::SocketAddrV4;

#[tokio::main]
pub async fn main() -> Result<()> {
    let connection_path = EPath::from(vec![Segment::Port(PortSegment::default())]);
    let endpoint = SocketAddrV4::new("192.168.0.83".parse()?, EIP_DEFAULT_PORT);
    let mut client = EipClient::new(endpoint).with_connection_path(connection_path);
    let mr_request = MessageRouterRequest::new(
        0x4c,
        EPath::from(vec![Segment::Symbol("test_car1_x".to_owned())]),
        ElementCount(1),
    );
    let resp = client.send(mr_request).await?;
    assert_eq!(resp.reply_service, 0xCC); // read tag service reply
    assert_eq!(LittleEndian::read_u16(&resp.data[0..2]), 0xC4); // DINT
    client.close().await?;
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
