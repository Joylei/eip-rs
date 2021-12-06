// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

/*!

# rseip

rseip - EIP&CIP in pure Rust

## Features

- Pure Rust Library
- Asynchronous
- Extensible
- Explicit Messaging (Connected / Unconnected)
- Open Source

## How to use

Add `rseip` to your cargo project's dependencies

```toml
rseip={git="https://github.com/Joylei/eip-rs.git"}
```

## Example

### Read tag from Allen-bradley CompactLogIx device

```rust,no_run
use anyhow::Result;
use rseip::{
    cip::service::MessageService,
    cip::{connection::Options, epath::EPath},
    client::{ab_eip::{PathParser, TagValue}, AbEipConnection, AbService},
};

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = AbEipConnection::new_host_lookup("192.168.0.83", Options::default()).await?;
    let tag = EPath::parse_tag("test_car1_x")?;
    println!("read tag...");
    let value: TagValue = client.read_tag(tag.clone()).await?;
    println!("tag value: {:?}", value);
    client.write_tag(tag, value).await?;
    println!("write tag - done");
    client.close().await?;
    Ok(())
}
```

Please find more examples within [examples](https://github.com/Joylei/eip-rs/tree/main/examples).

## License

MIT

*/

//#![warn(missing_docs)]

#![allow(clippy::match_like_matches_macro)]

/// adapters
pub mod adapters;
/// client
pub mod client;
mod error;

#[doc(inline)]
pub use error::ClientError;
pub use rseip_cip as cip;
/// library result
pub type Result<T> = core::result::Result<T, ClientError>;

pub use core::result::Result as StdResult;

#[cfg(test)]
mod test {
    use std::future::Future;

    #[allow(unused)]
    #[inline]
    pub(crate) fn block_on<F>(f: F)
    where
        F: Future<Output = anyhow::Result<()>>,
    {
        let mut builder = tokio::runtime::Builder::new_current_thread();
        builder.enable_all();
        let rt = builder.build().unwrap();
        rt.block_on(f).unwrap();
    }
}
