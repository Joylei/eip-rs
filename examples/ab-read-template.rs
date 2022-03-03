// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use anyhow::Result;
use futures_util::{future, StreamExt, TryStreamExt};
use rseip::client::ab_eip::*;
use rseip::precludes::*;

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = AbEipClient::new_host_lookup("192.168.0.83")
        .await?
        .with_connection_path(PortSegment::default());

    let instance_id = 2336;
    // here use a known instance_id, please uncomment below line to fetch one from PLC controller.
    //let instance_id = first_struct_instance(&mut client).await?.unwrap();
    let template = client.find_template(instance_id).await?;
    println!("template instance:\n{:?}", template);
    {
        let mut req = client.read_template(&template);
        let info = req.call().await?;
        println!("template definition:\n{:?}", info);
    }
    client.close().await?;
    Ok(())
}

#[allow(unused)]
async fn first_struct_instance(client: &mut AbEipClient) -> Result<Option<u16>> {
    let stream = client.list_tag().call();
    tokio::pin!(stream);
    let res = stream
        .try_filter_map(|item| future::ready(Ok(item.symbol_type.instance_id())))
        .next()
        .await;
    match res {
        Some(Ok(v)) => Ok(Some(v)),
        Some(Err(e)) => Err(e.into()),
        None => Ok(None),
    }
}
