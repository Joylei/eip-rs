// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use anyhow::Result;
use futures_util::StreamExt;
use rseip::{cip::identity::IdentityObject, client::EipDiscovery};
use std::time::Duration;

#[tokio::main]
pub async fn main() -> Result<()> {
    let stream = EipDiscovery::new("192.168.0.22".parse()?)
        .repeat(3)
        .interval(Duration::from_secs(3))
        .run::<IdentityObject>()
        .await?;

    stream
        .for_each(|item| {
            println!("{:?}", item);
            futures_util::future::ready(())
        })
        .await;

    Ok(())
}
