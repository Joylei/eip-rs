// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use rseip_core::Error;

#[async_trait::async_trait(?Send)]
pub trait Heartbeat {
    type Error: Error;
    /// send Heartbeat message to keep underline transport alive
    async fn heartbeat(&mut self) -> Result<(), Self::Error>;
}
