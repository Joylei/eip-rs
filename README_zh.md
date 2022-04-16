[EN](./README.md) | [中文](./README_zh.md)

# rseip
[![crates.io](https://img.shields.io/crates/v/rseip.svg)](https://crates.io/crates/rseip)
[![docs](https://docs.rs/rseip/badge.svg)](https://docs.rs/rseip)
[![build](https://github.com/joylei/eip-rs/workflows/build/badge.svg?branch=main)](https://github.com/joylei/eip-rs/actions?query=workflow%3A%22build%22)
[![license](https://img.shields.io/crates/l/rseip.svg)](https://github.com/joylei/eip-rs/blob/master/LICENSE)

纯Rust语言写的CIP客户端，支持CIP和AB PLC

## 特性

- 纯Rust语言
- 异步支持
- 偏好静态派发
- 可扩展
- 显式CIP消息（无连接或有连接）
- 开源

### 支持AB PLC的服务

- TAG 读操作
- TAG 写操作
- TAG 分片写操作 (Tag Read Fragmented)
- TAG 分片写操作 (Tag Write Fragmented)
- TAG 读修改写操作 (Tag Read Modify Write)
- TAG 遍历 (Tag List)
- 读取模板

## 怎么安装

添加 `rseip` 到 `cargo` 项目的依赖

```toml
rseip="0.3"
```

继续往下查看更多示例和帮助。


## 示例

### AB PLC Tag 读写

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

请到这里查看更多示例代码[examples](https://github.com/Joylei/eip-rs/tree/main/examples).

## 指南

### 快速开始

添加 `rseip` 到 `cargo` 项目的依赖

```toml
rseip="0.3"
```

然后，导入 `rseip` 的模块到项目中
```rust
use rseip::client::ab_eip::*;
use rseip::precludes::*;
```

然后， 创建一个无连接的客户端
```rust
let mut client = AbEipClient::new_host_lookup("192.168.0.83")
    .await?
    .with_connection_path(PortSegment::default());
```

或者， 创建一个有连接的客户端
```rust
let mut client =
    AbEipConnection::new_host_lookup("192.168.0.83", OpenOptions::default()).await?;
```

#### 读 TAG
```rust
let tag = EPath::parse_tag("test_car1_x")?;
println!("read tag...");
let value: TagValue<i32> = client.read_tag(tag.clone()).await?;
```
#### 写 TAG
```rust
let tag = EPath::parse_tag("test_car1_x")?;
let value = TagValue {
  tag_type: TagType::Dint,
  value: 10_i32,
};
client.write_tag(tag, value).await?;
println!("write tag - done");
```

### 关于 `TagValue`, `Decode`, and `Encode`

AB PLC 有原子类型， 结构类型，以及数组。本项目提供了 `Encode` 来编码数据，`Decode`来解码数据，以及`TagValue`来辅助数据值的操作。此项目已经为以下类型实现了`Encode` and `Decode`： `bool`,`i8`,`u8`,`i16`,`u16`,`i32`,`u32`,`i64`,`u64`,`f32`,`f64`,`i128`,`u128`,`()`,`Option`,`Tuple`,`Vec`,`[T;N]`,`SmallVec`。对于结构类型的数据你应该自己实现`Encode` and `Decode`。

#### 读数据

想要读取单个值（原子类型或结构类型），并且你知道要映射到的类型，可以这样操作：
```rust
let value: TagValue<MyType> = client.read_tag(tag).await?;
println!("{:?}",value);
```

想要读取 TAG 值类型，并且你不关心数据值，可以这样操作：
```rust
let value: TagValue<()> = client.read_tag(tag).await?;
println!("{:?}",value.tag_type);
```

想要读取原始的字节数据，可以这样操作：
```rust
let value: TagValue<Bytes> = client.read_tag(tag).await?;
```

想要遍历读取的数据值，可以这样操作：
```rust
let iter: TagValueTypedIter<MyType> = client.read_tag(tag).await?;
println!("{:?}", iter.tag_type());
while let Some(res) = iter.next(){
  println!("{:?}", res);
}
```

想要遍历读取的数据值，并且不知道具体类型，可以这样操作：
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

想要读取数组的多个元素，可以这样操作:
```rust
let value: TagValue<Vec<MyType>> = client.read_tag((tag,5_u16)).await?;
println!("{:?}",value);
```

#### 写数据

如果想要写数据，你必须知道TAG的类型。通常情况下，可以通过读的方式获得。但是对于结构类型，你不能持久的依赖所获得类型(`structure handle`），因为它是一个计算值(CRC)，当结构的定义变化时，这个值可能改变。

想要写入单个值（原子类型或结构类型），可以这样操作：
```rust
let value = TagValue {
  tag_type: TagType::Dint,
  value: 10_i32,
};
client.write_tag(tag, value).await?;
```

想要写入字节数据，可以这样操作：
```rust
let bytes:&[u8] = &[0,1,2,3];
let value = TagValue {
  tag_type: TagType::Dint,
  value: bytes,
};
client.write_tag(tag, value).await?;
```
也支持写入`bytes::Bytes`。


想要向数组写入多个数据元素，可以这样操作：
```rust
let items: Vec<MyType> = ...;
let value = TagValue {
  tag_type: TagType::Dint,
  value: items,
};
client.write_tag(tag, value).await?;
```
也支持写入`[T;N]`。


### 此外

由于一些考虑，`TagValue`并不支持任意实现了`Encode` or `Decode`的类型的读写。

但是你可以不使用 `TagValue`，可以定义自己的值编解码器，只要实现了`Encode`和`Decode`。由于`Encode`需要计算字节数，自己实现的编码器比通用的实现性能会更好。

对于简单场景来说，`Tuple`应该足够应对了。
```rust
let (tag_type,value):(TagType, i32) = client.read_tag(tag).await?;
client.write_tag(tag, (tag_type, 1_u16, value)).await?;
```

## 开源协议

MIT
