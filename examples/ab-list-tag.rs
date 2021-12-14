// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

//! list symbol instances

use anyhow::Result;
use futures_util::StreamExt;
use rseip::precludes::*;

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = AbEipClient::new_host_lookup("192.168.0.83")
        .await?
        .with_connection_path(PortSegment::default());
    {
        let stream = client.list_tag().call();
        stream
            .for_each(|item| async move {
                println!("{:?}", item);
            })
            .await;
    }
    client.close().await?;
    Ok(())
}
