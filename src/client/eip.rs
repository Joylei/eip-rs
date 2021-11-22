// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod discover;

use super::*;
use crate::{consts::EIP_DEFAULT_PORT, eip::context::EipContext};
pub use discover::EipDiscovery;
use futures_util::future::BoxFuture;
use std::{
    borrow::Cow,
    net::{SocketAddr, SocketAddrV4},
};
use tokio::net::{lookup_host, TcpSocket, TcpStream};

/// Generic EIP Client
pub type EipClient = Client<EipDriver>;

/// Generic EIP Connection
pub type EipConnection = Connection<EipDriver>;

/// Generic EIP driver
pub struct EipDriver;

impl Driver for EipDriver {
    type Endpoint = SocketAddrV4;
    type Service = EipContext<TcpStream>;

    fn build_service(addr: &Self::Endpoint) -> BoxFuture<Result<Self::Service>> {
        let addr = addr.clone();
        let fut = async move {
            let socket = TcpSocket::new_v4()?;
            let stream = socket.connect(addr.into()).await?;
            let service = EipContext::new(stream);
            Ok(service)
        };
        Box::pin(fut)
    }
}

impl EipClient {
    pub async fn new_host_lookup(host: impl AsRef<str>) -> io::Result<Self> {
        let addr = resolve_host(host).await?;
        Ok(Self::new(addr))
    }
}

impl EipConnection {
    pub async fn new_host_lookup(host: impl AsRef<str>, options: Options) -> io::Result<Self> {
        let addr = resolve_host(host).await?;
        Ok(Self::new(addr, options))
    }
}

pub(crate) async fn resolve_host(host: impl AsRef<str>) -> io::Result<SocketAddrV4> {
    let host: Cow<_> = {
        let host = host.as_ref();
        if host.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid host"));
        }
        if !host.contains(":") {
            Cow::Owned(format!("{}:{}", host, EIP_DEFAULT_PORT))
        } else {
            host.into()
        }
    };
    let addr = lookup_host(host.as_ref())
        .await?
        .filter_map(|item| match item {
            SocketAddr::V4(addr) => Some(addr),
            _ => None,
        })
        .next();
    if let Some(addr) = addr {
        Ok(addr)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "dns lookup failure",
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        cip::epath::{PortSegment, Segment},
        consts::EIP_DEFAULT_PORT,
        test::block_on,
    };
    use byteorder::{ByteOrder, LittleEndian};
    use bytes::{BufMut, BytesMut};

    #[test]
    fn ab_read_tag() {
        block_on(async {
            let connection_path = EPath::from(vec![Segment::Port(PortSegment::default())]);
            let mut client =
                EipClient::new_host_lookup("192.168.0.83").with_connection_path(connection_path);
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
        });
    }

    struct ElementCount(u16);

    impl Encodable for ElementCount {
        fn encode(self, dst: &mut BytesMut) -> Result<()> {
            dst.put_u16_le(self.0);
            Ok(())
        }
        fn bytes_count(&self) -> usize {
            2
        }
    }
}
