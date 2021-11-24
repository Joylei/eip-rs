// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::StdResult;

#[async_trait::async_trait(?Send)]
pub trait Heartbeat {
    type Error;
    /// send Heartbeat message to keep underline transport alive
    async fn heartbeat(&mut self) -> StdResult<(), Self::Error>;
}
