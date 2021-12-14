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
    let prev_value: TagValue<i32> = client.read_tag(tag.clone()).await?;
    println!("tag value: {:#02x?}", prev_value);
    client
        .write_tag(
            tag.clone(),
            TagValue {
                tag_type: TagType::Dint,
                value: 0x12_68_72_40_i32,
            },
        )
        .await?;
    println!("read tag...");
    let prev_value: (TagType, [u8; 4]) = client.read_tag(tag.clone()).await?;
    println!("tag value before read_modify_write: {:#02x?}", prev_value);
    let req = {
        let mut req = ReadModifyWriteRequest::<4>::new().tag(tag.clone());
        req.or_mask_mut()[0] = 0x04; // set bit 2
        req.and_mask_mut()[0] = 0xDF; // reset bit 5
        req
    };
    client.read_modify_write(req).await?;
    let value: (TagType, [u8; 4]) = client.read_tag(tag).await?;
    println!("value after read_modify_write: {:#02x?}", value);
    client.close().await?;
    Ok(())
}
