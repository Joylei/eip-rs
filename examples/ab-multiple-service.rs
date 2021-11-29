// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use anyhow::Result;
use rseip::{
    cip::{
        connection::Options,
        epath::EPath,
        service::{CommonServices, MessageService},
        MessageRequest,
    },
    client::{
        ab_eip::{ElementCount, TagValue},
        AbEipConnection,
    },
};
use std::convert::TryFrom;

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = AbEipConnection::new_host_lookup("192.168.0.83", Options::default()).await?;
    let mr = client
        .multiple_service()
        .push(MessageRequest::new(
            0x4C,
            EPath::from_symbol("test_car1_x"),
            ElementCount(1),
        ))
        .push(MessageRequest::new(
            0x4C,
            EPath::from_symbol("test_car2_x"),
            ElementCount(1),
        ));
    let iter = mr.send().await?;
    for item in iter {
        let item = item?;
        assert_eq!(item.reply_service, 0xCC);
        if item.status.is_err() {
            println!("error read tag: {}", item.status);
        } else {
            let value = TagValue::try_from(item.data);
            println!("tag value: {:?}", value);
        }
    }
    client.close().await?;
    Ok(())
}
