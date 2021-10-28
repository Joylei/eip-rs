use anyhow::Result;
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, Bytes, BytesMut};
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use rseip::{
    client::{client::State, Client},
    codec::{ClientCodec, Encodable},
    frame::{
        cip::{
            connection::{
                ConnectionType, Direction, ForwardCloseReply, ForwardCloseRequest,
                ForwardOpenConnectionParameters, ForwardOpenReply, ForwardOpenRequest,
                ForwardOpenSuccess, Priority, TransportClass, TriggerType,
            },
            *,
        },
        command::SendUnitData,
    },
    service::client::TcpService,
};
use std::{convert::TryInto, net::SocketAddr};
use tokio::net::{TcpStream, UdpSocket};
use tokio_util::udp::UdpFramed;

#[tokio::main]
pub async fn main() -> Result<()> {
    let connection_path = EPath::from(vec![Segment::Port(PortSegment::default())]);
    let client = Client::connect("192.168.0.83").await?;
    let session_handle = client.session_handle().unwrap();

    //build a CIP connection
    let mut context = client.into_inner();
    let parameters = cip_connect(&mut context, connection_path.clone()).await?;

    // build udp socket
    let addr_local: SocketAddr = "127.0.0.1:2222".parse()?;
    let server_addr: SocketAddr = "192.168.0.83:2222".parse()?;
    let udp = UdpSocket::bind(addr_local).await?;
    let mut service = UdpFramed::new(udp, ClientCodec::default());

    let mr_request = MessageRouterRequest::new(
        0x4c,
        EPath::from(vec![Segment::Symbol("test_car1_x".to_owned())]),
        ElementCount(1),
    );

    assert!(parameters.o_t_connection_id > 0);

    connected_send(
        &mut service,
        parameters.o_t_connection_id,
        session_handle,
        server_addr,
        mr_request,
    )
    .await?;

    let (packet, _) = service.next().await.unwrap()?;
    let mr_reply: ConnectedSendReply<Bytes> = packet.try_into()?;
    let resp = mr_reply.into_inner();
    assert_eq!(resp.reply_service, 0xCC); // read tag service reply
    assert_eq!(LittleEndian::read_u16(&resp.data[0..2]), 0xC4); // DINT

    cip_disconnect(&mut context, &parameters, connection_path.clone()).await?;
    context.unregister_session().await?;

    Ok(())
}

async fn cip_disconnect(
    context: &mut State<TcpStream>,
    parameters: &ForwardOpenSuccess,
    path: EPath,
) -> Result<()> {
    let request = ForwardCloseRequest {
        priority_time_ticks: 0x03,
        timeout_tick: 0xfa,
        connection_serial_number: parameters.connection_serial_number,
        originator_serial_number: parameters.originator_serial_number,
        originator_vender_id: parameters.originator_vendor_id,
        connection_path: path,
    };
    let reply = context.forward_close(request).await?;
    match reply {
        ForwardCloseReply::Fail(_) => {
            panic!("forward_open failed")
        }
        ForwardCloseReply::Success { .. } => {}
    }
    Ok(())
}

async fn cip_connect(context: &mut State<TcpStream>, path: EPath) -> Result<ForwardOpenSuccess> {
    let connection_serial_number: u16 = rand::thread_rng().gen_range(1..0xFFFF);
    let request = ForwardOpenRequest {
        priority_time_ticks: 0x03,
        timeout_ticks: 0xfa,
        o_t_connection_id: 0,
        t_o_connection_id: 0,
        connection_serial_number,
        vendor_id: 0xFF,
        originator_serial_number: 0xFFFFFFFF,
        timeout_multiplier: 3,
        o_t_rpi: 0x7A120, //500ms
        t_o_rpi: 0x7A120, //500ms
        o_t_connection_parameters: ForwardOpenConnectionParameters {
            redundant_owner: false,
            connection_type: ConnectionType::P2P,
            connection_size: 4000,
            variable_length: connection::VariableLength::Variable,
            priority: Priority::Scheduled,
        },
        t_o_connection_parameters: ForwardOpenConnectionParameters {
            redundant_owner: false,
            connection_type: ConnectionType::Multicast,
            connection_size: 4000,
            variable_length: connection::VariableLength::Variable,
            priority: Priority::Scheduled,
        },
        connection_path: path,
        ..Default::default()
    }
    .with_transport_direction(Direction::Client)
    .with_transport_class(TransportClass::Class3)
    .with_transport_trigger(TriggerType::Application);

    let reply = context.forward_open(request).await?;
    let parameters = match reply {
        ForwardOpenReply::Fail(_) => {
            panic!("forward_open failed")
        }
        ForwardOpenReply::Success { reply, .. } => reply,
    };
    Ok(parameters)
}

async fn connected_send<R: Encodable>(
    service: &mut UdpFramed<ClientCodec>,
    connection_id: u32,
    session_handle: u32,
    server_addr: SocketAddr,
    request: R,
) -> Result<()> {
    let command = SendUnitData {
        session_handle,
        connection_id,
        sequence_number: None,
        data: request,
    };
    service.send((command, server_addr)).await?;
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
