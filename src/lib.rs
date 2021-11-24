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
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, BytesMut};
use rseip::{client::Client, codec::Encodable, frame::cip::*};

#[tokio::main]
pub async fn main() -> Result<()> {
    let connection_path = EPath::from(vec![Segment::Port(PortSegment::default())]);
    let mut client = Client::connect("192.168.0.83").await?;
    let mr_request = MessageRouterRequest::new(
        0x4c,
        EPath::from_symbol("test_car1_x"),
        ElementCount(1),
    );
    let resp = client.send(mr_request, connection_path).await?;
    assert_eq!(resp.reply_service, 0xCC); // read tag service reply
    assert_eq!(LittleEndian::read_u16(&resp.data[0..2]), 0xC4); // DINT
    client.close().await?;
    Ok(())
}

struct ElementCount(u16);

impl Encodable for ElementCount {
    fn encode(self, dst: &mut BytesMut) -> rseip::Result<()> {
        dst.put_u16_le(self.0);
        Ok(())
    }
    fn bytes_count(&self) -> usize {
        2
    }
}

```

Please find more examples within [examples](https://github.com/Joylei/eip-rs/tree/main/examples).

## License

MIT

*/

pub mod adapters;
pub mod client;
mod error;

pub use error::ClientError as Error;
pub use rseip_cip as cip;
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod test {
    use std::future::Future;

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
