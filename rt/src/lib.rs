// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2023, Joylei <leingliu@gmail.com>
// License: MIT

/*!
# rseip-rt
rt module for `rseip`, please look at [rseip project](https://github.com/Joylei/eip-rs) for more information.

## License

MIT
*/

use futures_util::future::BoxFuture;
use std::{io, net::SocketAddrV4, time::Duration};
pub use udp::{AsyncUdpReadHalf, AsyncUdpSocket, AsyncUdpWriteHalf};
pub mod udp;

pub trait Runtime {
    type Transport;
    type UdpSocket: AsyncUdpSocket;
    type Sleep;
    fn lookup_host(host: String) -> BoxFuture<'static, io::Result<SocketAddrV4>>;
    fn sleep(duration: Duration) -> Self::Sleep;
}

#[cfg(feature = "rt-tokio")]
mod rt_tokio;

#[cfg(feature = "rt-smol")]
mod rt_smol;

#[cfg(all(feature = "rt-tokio", not(any(feature = "rt-smol"))))]
pub use rt_tokio::TokioRuntime as CurrentRuntime;

#[cfg(feature = "rt-smol")]
pub use rt_smol::SmolRuntime as CurrentRuntime;

#[cfg(not(any(feature = "rt-tokio", feature = "rt-smol")))]
compile_error!("feature \"rt-tokio\" or \"rt-smol\" must be enabled");
