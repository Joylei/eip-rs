// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use anyhow::Result;
use rseip::client::ab_eip::*;
use rseip::precludes::*;

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = AbEipClient::new_host_lookup("192.168.0.83")
        .await?
        .with_connection_path(PortSegment::default());
    let tag = EPath::parse_tag("test_car1_x")?;
    println!("read tag...");
    let value: TagValue<i32> = client.read_tag(tag.clone()).await?;
    println!("tag value: {:?}", value);
    client.write_tag(tag, value).await?;
    println!("write tag - done");
    client.close().await?;
    Ok(())
}
