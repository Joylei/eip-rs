// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2020-2021, Joylei <leingliu@gmail.com>
// License: MIT

pub mod client;
pub mod connection;
mod discovery;

pub use client::Client;
pub use connection::Connection;
pub use discovery::discover;
