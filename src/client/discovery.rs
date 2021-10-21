use futures_util::{SinkExt, Stream, StreamExt, TryStreamExt};
use std::io;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

use crate::codec::ClientCodec;
use crate::error::Error;
use crate::frame::{Request, Response};

pub async fn discover(
    listen_addr: SocketAddr,
    broadcast_addr: SocketAddr,
) -> Result<impl Stream<Item = (Response, SocketAddr)>, Error> {
    if listen_addr.is_ipv6() || broadcast_addr.is_ipv6() {
        return Err(io::Error::new(io::ErrorKind::Other, "ipv6 not supported").into());
    }

    let socket = UdpSocket::bind(listen_addr).await?;
    socket.set_broadcast(true)?;

    let mut service = UdpFramed::new(socket, ClientCodec::default());

    log::debug!("ListIdentity: sending request");
    service
        .send((Request::ListIdentity, broadcast_addr))
        .await?;
    log::debug!("ListIdentity: request sent");

    let stream = service.into_stream().filter_map(|res| async move {
        match res {
            Ok((Response::ListIdentity(items), addr)) => {
                log::debug!("ListIdentity: reply received from {}", addr);
                Some((Response::ListIdentity(items), addr))
            }
            Ok(_) => {
                log::debug!("ListIdentity: bad reply, ignore");
                None
            }
            Err(e) => {
                log::debug!("ListIdentity: reply err: {}", e);
                None
            }
        }
    });
    Ok(stream)
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::pin;

    #[test]
    fn test_discover() {
        let mut builder = tokio::runtime::Builder::new_current_thread();
        builder.enable_all();
        let rt = builder.build().unwrap();
        rt.block_on(async {
            let stream = discover(
                "192.168.0.22:0".parse().unwrap(),
                "192.168.0.255:44818".parse().unwrap(),
            )
            .await
            .unwrap();
            pin!(stream);
            let item = stream.next().await.unwrap();
            println!("{:?}", item)
        });
    }
}
