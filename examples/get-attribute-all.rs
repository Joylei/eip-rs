// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2023, Joylei <leingliu@gmail.com>
// License: MIT

use anyhow::Result;
use bytes::Buf;
use core::{slice, str};
use rseip::precludes::*;
use rseip_cip::Revision;
use std::borrow::Cow;

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = EipClient::new_host_lookup("192.168.0.83")
        .await?
        .with_connection_path(PortSegment::default());
    let identity_object_class = 0x01;
    let path = EPath::new().with_class(identity_object_class);
    // raw bytes
    //let value: BytesHolder = client.get_attribute_all(path).await?;
    //dbg!(value);
    let value: TheIdentity = client.get_attribute_all(path).await?;
    dbg!(value);
    client.close().await?;
    Ok(())
}

#[derive(Debug)]
struct TheIdentity<'a> {
    /// device manufacturers vendor id
    pub vendor_id: u16,
    /// device type of product
    pub device_type: u16,
    /// product code
    pub product_code: u16,
    /// device revision
    pub revision: Revision,
    /// current status of device
    pub status: u16,
    /// serial number of device
    pub serial_number: u32,
    /// short string
    pub product_name: Cow<'a, str>,
}

impl<'de> Decode<'de> for TheIdentity<'de> {
    fn decode<D>(mut decoder: D) -> rseip::StdResult<Self, D::Error>
    where
        D: rseip_core::codec::Decoder<'de>,
    {
        let identity = TheIdentity {
            vendor_id: decoder.decode_u16(),
            device_type: decoder.decode_u16(),
            product_code: decoder.decode_u16(),
            revision: Revision {
                major: decoder.decode_u8(),
                minor: decoder.decode_u8(),
            },
            status: decoder.decode_u16(),
            serial_number: decoder.decode_u32(),
            product_name: {
                let name_len = decoder.decode_u8();
                decoder.ensure_size(name_len as usize)?;
                let data = decoder.buf_mut().copy_to_bytes(name_len as usize);
                unsafe {
                    let buf = data.as_ptr();
                    let buf = slice::from_raw_parts(buf, name_len as usize);
                    let name = str::from_utf8_unchecked(buf);
                    Cow::from(name)
                }
            },
        };

        Ok(identity)
    }
}
