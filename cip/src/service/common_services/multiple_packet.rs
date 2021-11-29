// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{
    codec::{Encodable, LazyEncode},
    epath::EPath,
    service::*,
    *,
};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::convert::TryInto;
use rseip_core::InnerError;
use smallvec::SmallVec;

/// build and send multiple service packet
pub struct MultipleServicePacket<'a, T, P, D> {
    inner: &'a mut T,
    items: SmallVec<[MessageRequest<P, D>; 8]>,
}

impl<'a, T, P, D> MultipleServicePacket<'a, T, P, D> {
    pub(crate) fn new(inner: &'a mut T) -> Self {
        Self {
            inner,
            items: Default::default(),
        }
    }
}

impl<'a, T, P, D> MultipleServicePacket<'a, T, P, D>
where
    T: MessageService,
    P: Encodable,
    D: Encodable,
{
    /// append service request
    pub fn push(mut self, mr: MessageRequest<P, D>) -> Self {
        self.items.push(mr);
        self
    }

    /// append all service requests
    pub fn push_all(mut self, items: impl Iterator<Item = MessageRequest<P, D>>) -> Self
    where
        P: Encodable + 'static,
        D: Encodable + 'static,
    {
        for mr in items {
            self.items.push(mr);
        }
        self
    }

    /// build and send requests
    pub async fn send(
        self,
    ) -> StdResult<impl Iterator<Item = Result<MessageReply<Bytes>>>, T::Error> {
        let Self { inner, items } = self;
        if items.len() == 0 {
            return Ok(ReplyIter::End);
        }

        let start_offset = 2 + 2 * items.len();
        let bytes_count = items.iter().map(|v| v.bytes_count()).sum::<usize>() + start_offset;
        let mr = MessageRequest {
            service_code: 0x0A,
            path: EPath::default().with_class(2).with_instance(1),
            data: LazyEncode {
                f: |buf: &mut BytesMut| {
                    buf.put_u16_le(items.len() as u16);
                    let mut offset = start_offset;
                    for item in items.iter() {
                        buf.put_u16_le(offset as u16);
                        offset += item.bytes_count();
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

        if reply.reply_service != 0x8A {
            return Err(Error::from_invalid_data()
                .with_context(format!(
                    "unexpected reply service for multiple service packet: {:#0x}",
                    reply.reply_service
                ))
                .into());
        }

        let res = ReplyIter::Init(reply.data);
        Ok(res)
    }
}

enum ReplyIter {
    Init(Bytes),
    HasCount {
        buf: Bytes,
        count: u16,
    },
    HasOffset {
        buf: Bytes,
        data: Bytes,
        count: u16,
        last: Option<u16>,
        i: u16,
    },
    End,
}

impl ReplyIter {
    #[inline]
    fn raise_err<T>(&mut self) -> Option<Result<T>> {
        *self = Self::End;
        Some(Err(Error::from(InnerError::InvalidData)
            .with_context("CIP - failed to decode message reply")))
    }
}

impl Iterator for ReplyIter {
    type Item = Result<MessageReply<Bytes>>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self {
                Self::Init(buf) => {
                    if buf.len() < 2 {
                        return self.raise_err();
                    }
                    let count = buf.get_u16_le();
                    *self = if count == 0 {
                        Self::End
                    } else {
                        Self::HasCount {
                            buf: buf.clone(),
                            count,
                        }
                    };
                }
                Self::HasCount { buf, count } => {
                    let data_offsets = 2 * (*count) as usize;
                    if buf.len() < data_offsets {
                        return self.raise_err();
                    }
                    *self = Self::HasOffset {
                        buf: buf.clone(),
                        count: *count,
                        data: buf.split_off(data_offsets),
                        i: 0,
                        last: None,
                    };
                }
                Self::HasOffset {
                    buf,
                    data,
                    count,
                    last,
                    i,
                } => {
                    if i < count {
                        *i += 1;
                        let offset = buf.get_u16_le();
                        if let Some(last) = last.replace(offset) {
                            if offset <= last {
                                return self.raise_err();
                            }
                            let size = (offset - last) as usize;
                            if data.len() < size {
                                return self.raise_err();
                            }
                            let buf = data.split_to(size);
                            let res: Result<MessageReply<Bytes>> = buf.try_into();
                            return Some(res);
                        }
                        continue;
                    }
                    // process remaining
                    if data.len() > 0 {
                        let res: Result<MessageReply<Bytes>> = data.split_to(data.len()).try_into();
                        *self = Self::End;
                        return Some(res);
                    }
                    *self = Self::End;
                }
                Self::End => return None,
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Init(_) => (0, None),
            Self::HasCount { count, .. } => (0, Some(*count as usize)),
            Self::HasOffset { count, i, .. } => (0, Some((*count - *i) as usize)),
            Self::End => (0, Some(0)),
        }
    }
}
