use crate::{
    codec::{ClientCodec, Encodable},
    error::Error,
    frame::{cip::MessageRouterRequest, command::SendUnitData},
    Result,
};
use futures_util::{Sink, SinkExt, StreamExt};
use std::{collections::HashMap, net::SocketAddr};
use tokio_util::udp::UdpFramed;

use super::Client;

/// CIP connection
pub trait Context {
    type Sender;

    fn session(&self) -> &Client;
    fn session_mut(&self) -> &mut Client;
    fn connection_id(&self) -> u32;
    fn remote_addr(&self) -> SocketAddr;
    fn sender<Item>(&self) -> &mut Self::Sender;
}

pub type ConnectedSend<P, D> = SendUnitData<MessageRouterRequest<P, D>>;

#[async_trait::async_trait(?Send)]
pub trait Send: Context {
    #[inline]
    async fn send<P, D>(
        &mut self,
        mr: MessageRouterRequest<P, D>,
        sequence_number: Option<u32>,
    ) -> Result<()>
    where
        Self::Sender: Sink<(ConnectedSend<P, D>, SocketAddr), Error = Error> + Unpin,
        P: Encodable,
        D: Encodable,
    {
        let session_handle = self.session().session_handle().unwrap();
        let command = SendUnitData {
            connection_id: self.connection_id(),
            session_handle,
            sequence_number,
            data: mr,
        };
        let remote = self.remote_addr();
        self.sender::<(ConnectedSend<P, D>, SocketAddr)>()
            .send((command, remote))
            .await?;
        Ok(())
    }
}

struct Connection {}

struct ConnectionManager {
    clients: HashMap<u32, Client>,
    connections: HashMap<(SocketAddr, u32), Connection>,
}

impl ConnectionManager {
    pub async fn run(mut self) -> Result<()> {}
}

pub enum Message {
    SessionConnected {
        session: Client,
    },
    SessionClosed {
        session_id: u32,
    },
    ConnectionReady {
        session_id: u32,
        connection_id: u32,
        service: UdpFramed<ClientCodec>,
    },
    ConnectionLost {
        session_id: u32,
        connection_id: u32,
    },
}
