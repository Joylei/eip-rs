// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{
    cip::{epath::EPath, MessageReply, MessageRequest},
    codec::{
        encode::{DynamicEncode, LazyEncode},
        Encodable,
    },
    eip::EipError,
    service::*,
    Error, Result,
};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::convert::TryInto;

/// build and send multiple service packet
pub struct MultipleServicePacket<'a, T> {
    inner: &'a mut T,
    items: Vec<DynamicEncode>,
}

impl<'a, T: MessageRouter> MultipleServicePacket<'a, T> {
    pub(crate) fn new(inner: &'a mut T) -> Self {
        Self {
            inner,
            items: Default::default(),
        }
    }

    /// append service request
    pub fn push<P, D>(mut self, mr: MessageRequest<P, D>) -> Self
    where
        P: Encodable + 'static,
        D: Encodable + 'static,
    {
        let bytes_count = mr.bytes_count();
        self.items.push(DynamicEncode {
            f: Box::new(|buf| mr.encode(buf)),
            bytes_count,
        });
        self
    }

    /// append all service requests
    pub fn push_all<P, D>(mut self, items: impl IntoIterator<Item = MessageRequest<P, D>>) -> Self
    where
        P: Encodable + 'static,
        D: Encodable + 'static,
    {
        for mr in items {
            let bytes_count = mr.bytes_count();
            self.items.push(DynamicEncode {
                f: Box::new(|buf| mr.encode(buf)),
                bytes_count,
            });
        }
        self
    }

    /// build and send requests
    pub async fn send(self) -> Result<Vec<MessageReply<Bytes>>> {
        let Self { inner, items } = self;
        if items.len() == 0 {
            return Ok(Default::default());
        }

        let start_offset = 2 + 2 * items.len();
        let bytes_count = items.iter().map(|v| v.bytes_count).sum::<usize>() + start_offset;
        let mr = MessageRequest {
            service_code: 0x0A,
            path: EPath::default().with_class(2).with_instance(1),
            data: LazyEncode {
                f: |buf: &mut BytesMut| {
                    buf.put_u16_le(items.len() as u16);
                    let mut offset = start_offset;
                    for item in items.iter() {
                        buf.put_u16_le(offset as u16);
                        offset += item.bytes_count;
                    }
                    for item in items {
                        item.encode(buf)?;
                    }
                    Ok(())
                },
                bytes_count,
            },
        };
        let reply = inner.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        decode_replies(reply.data)
    }
}

fn decode_replies(mut buf: Bytes) -> Result<Vec<MessageReply<Bytes>>> {
    if buf.len() < 2 {
        return Err(Error::Eip(EipError::InvalidData));
    }
    let mut results = Vec::new();
    let count = buf.get_u16_le();
    if count == 0 {
        return Ok(results);
    }
    let data_offsets = 2 * count as usize;
    if buf.len() < data_offsets {
        return Err(Error::Eip(EipError::InvalidData));
    }
    let mut data = buf.split_off(data_offsets);
    let mut last = None;
    for _ in 0..count {
        let offset = buf.get_u16_le();
        if let Some(last) = last.replace(offset) {
            if offset <= last {
                return Err(Error::Eip(EipError::InvalidData));
            }
            let size = (offset - last) as usize;
            if data.len() < size {
                return Err(Error::Eip(EipError::InvalidData));
            }
            let buf = data.split_to(size);
            let reply: MessageReply<Bytes> = buf.try_into()?;
            results.push(reply);
        }
    }

    // process remaining
    if data.len() > 0 {
        let reply: MessageReply<Bytes> = data.try_into()?;
        results.push(reply);
    }

    Ok(results)
}
