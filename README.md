# rseip

rseip - EIP&CIP in pure Rust

Note: still under development. Any part of the code might change from time to time.

## Features

- Pure Rust Library
- Asynchronous
- Prefer static dispatch
- Extensible
- Explicit Messaging (Connected / Unconnected)
- Open Source

### Services Supported for AB PLC

- Read Tag
- Write Tag
- Read Tag Fragmented
- Write Tag Fragmented
- Read Modify Write Tag
- Get Instance Attribute List
- Read Template

## How to use

Add `rseip` to your cargo project's dependencies

```toml
rseip={git="https://github.com/Joylei/eip-rs.git"}
```

## Example

### Tag Read/Write for Allen-bradley CompactLogIx device

```rust
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


## Related Projects

- [EIPScanner](https://github.com/nimbuscontrols/EIPScanner)

   Free implementation of EtherNet/IP in C++

- [EEIP.NET](https://github.com/rossmann-engineering/EEIP.NET)

  Ethernet/IP compatible library for .NET implementations

- [ digitalpetri/ethernet-ip](https://github.com/digitalpetri/ethernet-ip)
  
  Asynchronous, non-blocking, EtherNet/IP client implementation for Java

- [node-ethernet-ip](https://github.com/cmseaton42/node-ethernet-ip)

  A Lightweight Ethernet/IP API written to interface with Rockwell ControlLogix/CompactLogix Controllers. 

- [OpENer](https://github.com/EIPStackGroup/OpENer)
   
  OpENer is an EtherNet/IP stack for I/O adapter devices. It supports multiple I/O and explicit connections and includes objects and services for making EtherNet/IP-compliant products as defined in the ODVA specification. 

- [cpppo](https://github.com/pjkundert/cpppo/)
  
  Communications Protocol Python Parser and Originator -- EtherNet/IP CIP
