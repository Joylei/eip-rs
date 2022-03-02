// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

#![allow(unused)]

use anyhow::Result;
use bytes::{BufMut, BytesMut};
use rseip::precludes::*;
use rseip::{client::ab_eip::*, ClientError};
use rseip_core::codec::{Decoder, LittleEndianDecoder};

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client =
        AbEipConnection::new_host_lookup("192.168.0.83", OpenOptions::default()).await?;
    let tag = EPath::parse_tag("test_frag")?;
    println!("read tag...");
    let mut buf = BytesMut::new();
    let mut tag_type = None;
    loop {
        let req = ReadFragmentedRequest::new()
            .tag(tag.clone())
            .offset(buf.len() as u16);
        let (has_more, value) = client.read_tag_fragmented(req).await.unwrap();
        tag_type = Some(value.tag_type);
        let bytes = value.value;
        dbg!(bytes.len());
        buf.put_slice(&bytes[..]);
        if !has_more {
            break;
        }
    }
    println!("buf.len: {}", buf.len());
    let mut decoder = LittleEndianDecoder::<ClientError>::new(buf.freeze());
    let udt: BigUdt = decoder.decode_any()?;
    println!("tag type: {:?}", tag_type);
    println!("tag value: {:?}", udt);

    client.close().await?;
    Ok(())
}

const DEFAULT_STRING_CAPACITY: usize = 82;
/// total bytes = 4 + 82
#[derive(Debug, Default)]
struct AbString {
    data: String,
}

impl Encode for AbString {
    fn encode_by_ref<A: rseip_core::codec::Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        let mut data = self.data.as_bytes();
        if data.len() > DEFAULT_STRING_CAPACITY {
            data = &data[0..DEFAULT_STRING_CAPACITY];
        }
        // LEN
        buf.put_u16_le(data.len() as u16);
        buf.put_u16_le(0);
        //DATA
        let remaining = DEFAULT_STRING_CAPACITY - data.len();
        buf.put_slice(data);
        if remaining > 0 {
            buf.put_bytes(0, remaining);
        }
        Ok(())
    }

    fn bytes_count(&self) -> usize {
        4 + DEFAULT_STRING_CAPACITY
    }
}

impl<'de> Decode<'de> for AbString {
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: rseip_core::codec::Decoder<'de>,
    {
        decoder.ensure_size(4 + DEFAULT_STRING_CAPACITY)?;
        let len = decoder.decode_u16() as usize;
        let _ = decoder.decode_u16();
        let mut data = [0; DEFAULT_STRING_CAPACITY];
        let mut i = 0;
        while i < len {
            data[i] = decoder.decode_u8();
            i += 1;
        }
        Ok(Self {
            data: String::from_utf8_lossy(&data[0..len]).to_string(),
        })
    }
}

/// total bytes = (4 + 82) * 16 = 1408
#[derive(Debug, Default)]
struct BigUdt {
    member1: AbString,
    member2: AbString,
    member3: AbString,
    member4: AbString,
    member5: AbString,
    member6: AbString,
    member7: AbString,
    member8: AbString,
    member9: AbString,
    member10: AbString,
    member11: AbString,
    member12: AbString,
    member13: AbString,
    member14: AbString,
    member15: AbString,
    member16: AbString,
}

impl Encode for BigUdt {
    fn encode_by_ref<A: rseip_core::codec::Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        self.member1.encode_by_ref(buf, encoder)?;
        self.member2.encode_by_ref(buf, encoder)?;
        self.member3.encode_by_ref(buf, encoder)?;
        self.member4.encode_by_ref(buf, encoder)?;
        self.member5.encode_by_ref(buf, encoder)?;
        self.member6.encode_by_ref(buf, encoder)?;
        self.member8.encode_by_ref(buf, encoder)?;
        self.member9.encode_by_ref(buf, encoder)?;
        self.member10.encode_by_ref(buf, encoder)?;
        self.member11.encode_by_ref(buf, encoder)?;
        self.member12.encode_by_ref(buf, encoder)?;
        self.member13.encode_by_ref(buf, encoder)?;
        self.member14.encode_by_ref(buf, encoder)?;
        self.member15.encode_by_ref(buf, encoder)?;
        self.member16.encode_by_ref(buf, encoder)?;
        Ok(())
    }

    fn bytes_count(&self) -> usize {
        16 * 86
    }
}

impl<'de> Decode<'de> for BigUdt {
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: rseip_core::codec::Decoder<'de>,
    {
        let mut res = Self::default();
        res.member1 = decoder.decode_any()?;
        res.member2 = decoder.decode_any()?;
        res.member3 = decoder.decode_any()?;
        res.member4 = decoder.decode_any()?;
        res.member5 = decoder.decode_any()?;
        res.member6 = decoder.decode_any()?;
        res.member7 = decoder.decode_any()?;
        res.member8 = decoder.decode_any()?;
        res.member9 = decoder.decode_any()?;
        res.member10 = decoder.decode_any()?;
        res.member11 = decoder.decode_any()?;
        res.member12 = decoder.decode_any()?;
        res.member13 = decoder.decode_any()?;
        res.member14 = decoder.decode_any()?;
        res.member15 = decoder.decode_any()?;
        res.member16 = decoder.decode_any()?;
        Ok(res)
    }
}
