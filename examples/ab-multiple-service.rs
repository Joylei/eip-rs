// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use anyhow::Result;
use rseip::{
    cip::{connection::OpenOptions, MessageReply, MessageRequest, REPLY_MASK},
    client::ab_eip::*,
    precludes::*,
};

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client =
        AbEipConnection::new_host_lookup("192.168.0.83", OpenOptions::default()).await?;
    let mr = client
        .multiple_service()
        .push(MessageRequest::new(
            SERVICE_READ_TAG,
            EPath::parse_tag("test_car1_x")?,
            1_u16, // number of elements to read, u16
        ))
        .push(MessageRequest::new(
            SERVICE_READ_TAG,
            EPath::parse_tag("test_car2_x")?,
            1_u16, // number of elements to read, u16
        ));
    let mut iter = mr.call().await?;
    while let Some(item) = iter.next() {
        let item: MessageReply<TagValue<i32>> = item?;
        assert_eq!(item.reply_service, SERVICE_READ_TAG + REPLY_MASK);
        if item.status.is_err() {
            println!("error read tag: {}", item.status);
        } else {
            let value = item.data;
            println!("tag value: {:?}", value);
        }
    }
    client.close().await?;
    Ok(())
}
