// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use anyhow::Result;
use rseip::{
    cip::{connection::Options, epath::EPath, service::MessageService},
    client::{ab_eip::TagValue, AbEipConnection, AbService},
};

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = AbEipConnection::new_host_lookup("192.168.0.83", Options::default()).await?;
    let tag = EPath::from_symbol("test_car1_x");
    println!("read tag...");
    let value: TagValue = client.read_tag(tag.clone()).await?;
    println!("tag value: {:?}", value);
    client.write_tag(tag, value).await?;
    println!("write tag - done");
    client.close().await?;
    Ok(())
}
