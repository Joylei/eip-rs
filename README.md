[EN](./README.md) | [中文](./README_zh.md)

# rseip
[![crates.io](https://img.shields.io/crates/v/rseip.svg)](https://crates.io/crates/rseip)
[![docs](https://docs.rs/rseip/badge.svg)](https://docs.rs/rseip)
[![build](https://github.com/joylei/eip-rs/workflows/build/badge.svg?branch=main)](https://github.com/joylei/eip-rs/actions?query=workflow%3A%22build%22)
[![license](https://img.shields.io/crates/l/rseip.svg)](https://github.com/joylei/eip-rs/blob/master/LICENSE)

Ethernet/IP (CIP) client in pure Rust, for generic CIP and AB PLC

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
- Get Instance Attribute List (list tag)
- Read Template

## How to use

Add `rseip` to your cargo project's dependencies

```toml
rseip="0.3"
```

Please find detailed guides and examples from below sections.


## Example

### Tag Read/Write for Allen-bradley CompactLogIx device

```rust
use anyhow::Result;
use rseip::client::ab_eip::*;
use rseip::precludes::*;

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = AbEipClient::new_host_lookup("192.168.0.83")
        .await?
        .with_connection_path(PortSegment::default());
    let tag = EPath::parse_tag("test_car1_x")?;
    println!("read tag...");
    let value: TagValue<i32> = client.read_tag(tag.clone()).await?;
    println!("tag value: {:?}", value);
    client.write_tag(tag, value).await?;
    println!("write tag - done");
    client.close().await?;
    Ok(())
}
```

Please find more examples within [examples](https://github.com/Joylei/eip-rs/tree/main/examples).

## Guides
### Quick start

Add `rseip` to your cargo project's dependencies

```toml
rseip="0.3"
```

Then, import modules of `rseip` to your project
```rust
use rseip::client::ab_eip::*;
use rseip::precludes::*;
```

Then, create an unconnected client
```rust
let mut client = AbEipClient::new_host_lookup("192.168.0.83")
    .await?
    .with_connection_path(PortSegment::default());
```

or create a connection
```rust
let mut client =
    AbEipConnection::new_host_lookup("192.168.0.83", OpenOptions::default()).await?;
```

#### Read from a tag
```rust
let tag = EPath::parse_tag("test_car1_x")?;
println!("read tag...");
let value: TagValue<i32> = client.read_tag(tag.clone()).await?;
```
#### Write to a tag
```rust
let tag = EPath::parse_tag("test_car1_x")?;
let value = TagValue {
  tag_type: TagType::Dint,
  value: 10_i32,
};
client.write_tag(tag, value).await?;
println!("write tag - done");
```

### About `TagValue`, `Decode`, and `Encode`

As you may know, there are atomic types, structure types, and array type of tags. The library provides `Encode` to encode values, `Decode` to decode values, and `TagValue` to manipulate tag data values. The library already implements `Encode` and `Decode` for some rust types: `bool`,`i8`,`u8`,`i16`,`u16`,`i32`,`u32`,`i64`,`u64`,`f32`,`f64`,`i128`,`u128`,`()`,`Option`,`Tuple`,`Vec`,`[T;N]`,`SmallVec`. For structure type, you need to implement `Encode` and `Decode` by yourself.

#### Read

To get a single value (atomic/structure), and you know the exact mapped type, do like this
```rust
let value: TagValue<MyType> = client.read_tag(tag).await?;
println!("{:?}",value);
```

To get the tag type, and you do not care about the data part, do like this:
```rust
let value: TagValue<()> = client.read_tag(tag).await?;
println!("{:?}",value.tag_type);
```

To get the raw bytes whatever the data part holds, do like this:
```rust
let value: TagValue<Bytes> = client.read_tag(tag).await?;
```

To iterate values, and you know the exact mapped type, do like this:
```rust
let iter: TagValueTypedIter<MyType> = client.read_tag(tag).await?;
println!("{:?}", iter.tag_type());
while let Some(res) = iter.next(){
  println!("{:?}", res);
}
```

To iterate values, and you do not know the exact mapped type, do like this:
```rust
let iter: TagValueIter = client.read_tag(tag).await?;
println!("{:?}", iter.tag_type());
let res = iter.next::<bool>().unwrap();
println!("{:?}", res);
let res = iter.next::<i32>().unwrap();
println!("{:?}", res);
let res = iter.next::<MyType>().unwrap();
println!("{:?}", res);
```

To read more than 1 elements of an `Array`, do like this:
```rust
let value: TagValue<Vec<MyType>> = client.read_tag((tag,5_u16)).await?;
println!("{:?}",value);
```

#### Write

You must provide the tag type before you write to a tag. Normally, you can retrieve it by reading the tag. For structure type, you cannot reply on or persist the tag type (so called `structure handle`), it might change because it is a calculated value (CRC based).

To write a single value (atomic/structure), do like this:
```rust
let value = TagValue {
  tag_type: TagType::Dint,
  value: 10_i32,
};
client.write_tag(tag, value).await?;
```

To write raw bytes, do like this:
```rust
let bytes:&[u8] = &[0,1,2,3];
let value = TagValue {
  tag_type: TagType::Dint,
  value: bytes,
};
client.write_tag(tag, value).await?;
```

To write multiple values to an array, do like this:
```rust
let items: Vec<MyType> = ...;
let value = TagValue {
  tag_type: TagType::Dint,
  value: items,
};
client.write_tag(tag, value).await?;
```

### Moreover

For some reasons, `TagValue` does not work for all type that implements `Encode` or `Decode`.

But you can work without `TagValue`. You can define your own value holder, as long as it implements `Encode` and `Decode`.

For simple cases, `Tuple` should be a good option.
```rust
let (tag_type,value):(TagType, i32) = client.read_tag(tag).await?;
client.write_tag(tag, (tag_type, 1_u16, value)).await?;
```

## License

MIT
