// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2023, Joylei <leingliu@gmail.com>
// License: MIT

use anyhow::Result;
use bytes::Buf;
use core::{slice, str};
use rseip::precludes::*;

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = EipClient::new_host_lookup("192.168.0.83")
        .await?
        .with_connection_path(PortSegment::default());
    let identity_object_class = 0x01;
    let attr_id = 0x07; // product name
    let mut path = EPath::new()
        .with_class(identity_object_class)
        .with_instance(1);
    path.push(Segment::Attribute(attr_id));
    // raw bytes
    // let value: BytesHolder = client.get_attribute_single(path).await?;
    // dbg!(value);
    let value: ProductName = client.get_attribute_single(path).await?;
    dbg!(value);
    client.close().await?;
    Ok(())
}

#[derive(Debug)]
struct ProductName(String);

impl<'de> Decode<'de> for ProductName {
    fn decode<D>(mut decoder: D) -> rseip::StdResult<Self, D::Error>
    where
        D: rseip_core::codec::Decoder<'de>,
    {
        let name_len = decoder.decode_u8();
        decoder.ensure_size(name_len as usize)?;
        dbg!(name_len);
        let data = decoder.buf_mut().copy_to_bytes(name_len as usize);
        let product_name = unsafe {
            let buf = data.as_ptr();
            let buf = slice::from_raw_parts(buf, name_len as usize);
            let name = str::from_utf8_unchecked(buf);
            name.to_string()
        };

        Ok(Self(product_name))
    }
}
