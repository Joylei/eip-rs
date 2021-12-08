// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

pub mod message_reply;

use crate::*;
use crate::{identity::IdentityObject, socket::SocketAddr};
use bytes::Buf;
use rseip_core::codec::{Decode, Decoder};

impl<'de> Decode<'de> for IdentityObject {
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: rseip_core::codec::Decoder<'de>,
    {
        decoder.ensure_size(33)?;
        //let product_name_len = data[32];

        let identity = IdentityObject {
            protocol_version: decoder.decode_u16(),
            socket_addr: {
                let addr = decoder.buf_mut().copy_to_bytes(16);
                SocketAddr::from_bytes::<D::Error>(addr)?
            },
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
                decoder.ensure_size(name_len as usize + 1)?;
                let data = decoder.buf_mut().copy_to_bytes(name_len as usize);
                String::from_utf8_lossy(&data).into_owned().into()
            },
            state: decoder.decode_u8(),
        };

        Ok(identity)
    }
}

impl<'de> Decode<'de> for ListServiceItem {
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        debug_assert!(decoder.remaining() > 4);

        let item = ListServiceItem {
            protocol_version: decoder.decode_u16(),
            capability: decoder.decode_u16(),
            name: {
                decoder.ensure_size(16)?;
                let data = decoder.buf_mut().copy_to_bytes(16);
                String::from_utf8_lossy(&data).into_owned()
            },
        };

        Ok(item)
    }
}
