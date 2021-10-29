# rseip

rseip (eip-rs) - EtherNet/IP in pure Rust

## Features

- Pure Rust Library
- Asynchronous
- Extensible
- Open Source

## Work In Progress

- [x] ListIdentity
- [x] RegisterSession
- [x] UnregisterSession
- [x] SendRRData
- [x] UnconnectedSend
- [ ] ForwardOpen
- [ ] ForwardClose
- [ ] SendUnitData 

## How to use

Add `rseip` to your cargo project's dependencies

```toml
rseip={git="https://github.com/Joylei/eip-rs.git"}
```

## Example

### Read tag from Allen-bradley device

```rust
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
        EPath::from(vec![Segment::Symbol("test_car1_x".to_owned())]),
        ElementCount(1),
    );
    let resp = client.send(mr_request, connection_path).await?;
    assert_eq!(resp.reply_service, 0xCC); // read tag service reply
    assert_eq!(LittleEndian::read_u16(&resp.data[0..2]), 0xC4); // DINT
    client.close().await?;
    Ok(())
}
```

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
