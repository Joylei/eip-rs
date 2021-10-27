use crate::{
    codec::ClientCodec,
    frame::{command::ListIdentity, command_reply::ListIdentityReply},
    Result,
};
use futures_util::{SinkExt, Stream, StreamExt, TryStreamExt};
use std::{convert::TryInto, io, net::SocketAddr};
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

pub async fn discover(
    listen_addr: SocketAddr,
    broadcast_addr: SocketAddr,
) -> Result<impl Stream<Item = (ListIdentityReply, SocketAddr)>> {
    if listen_addr.is_ipv6() || broadcast_addr.is_ipv6() {
        return Err(io::Error::new(io::ErrorKind::Other, "ipv6 not supported").into());
    }

    let socket = UdpSocket::bind(listen_addr).await?;
    socket.set_broadcast(true)?;

    let mut service = UdpFramed::new(socket, ClientCodec::default());

    log::debug!("ListIdentity: sending request");
    service.send((ListIdentity, broadcast_addr)).await?;
    log::debug!("ListIdentity: request sent");

    let stream = service.into_stream().filter_map(|res| async move {
        match res {
            Ok((resp, addr)) => {
                let res: Result<ListIdentityReply> = resp.try_into();
                match res {
                    Ok(item) => {
                        log::debug!("ListIdentity: reply received from {}", addr);
                        Some((item, addr))
                    }
                    Err(e) => {
                        log::debug!("ListIdentity: reply err: {}", e);
                        None
                    }
                }
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
    use crate::test::block_on;
    use tokio::pin;

    //#[test]
    fn test_discover() {
        block_on(async {
            let stream =
                discover("192.168.0.22:0".parse()?, "192.168.0.255:44818".parse()?).await?;
            pin!(stream);
            let item = stream.next().await.unwrap();
            println!("{:?}", item);

            Ok(())
        });
    }
}
