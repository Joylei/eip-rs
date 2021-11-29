use crate::error::InnerError;
use crate::{Error, Result};
use bytes::Buf;
use bytes::Bytes;
use futures_util::stream;
use futures_util::Stream;
use rseip_cip::{epath::EPath, service::MessageService, CipError, MessageRequest};
use std::convert::TryFrom;

use super::HasMore;

/// symbol instance
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SymbolInstance {
    /// instance id
    pub id: u16,
    /// symbol name
    pub name: String,
    /// symbol data type
    pub symbol_type: u16,
}

pub struct GetInstanceAttributeList<'a, T> {
    inner: &'a mut T,
    start_instance: u16,
    all: bool,
}

impl<'a, T> GetInstanceAttributeList<'a, T> {
    pub(crate) fn new(inner: &'a mut T) -> Self {
        Self {
            inner,
            start_instance: 0,
            all: true,
        }
    }

    /// with starting instance id
    pub fn start_instance(mut self, instance_id: u16) -> Self {
        self.start_instance = instance_id;
        self
    }

    /// continue to send request if reply general status indicates more data to read
    pub fn retrieve_all(mut self, all: bool) -> Self {
        self.all = all;
        self
    }
}

impl<'a, T: MessageService<Error = Error>> GetInstanceAttributeList<'a, T> {
    pub fn call(self) -> impl Stream<Item = Result<SymbolInstance>> + 'a {
        let all = self.all;
        stream::try_unfold(
            State::Request {
                ctx: self.inner,
                start_instance: self.start_instance,
            },
            move |mut state| async move {
                loop {
                    match state {
                        State::Request {
                            ctx,
                            start_instance,
                        } => {
                            match get_attribute_list(ctx, start_instance.clone()).await {
                                Ok((has_more, data)) => {
                                    state = State::HasData {
                                        ctx,
                                        start_instance,
                                        has_more,
                                        data,
                                    }
                                }
                                Err(e) => {
                                    //state = State::End;
                                    return Err(e);
                                }
                            }
                        }
                        State::HasData {
                            ctx,
                            start_instance,
                            has_more,
                            mut data,
                        } => {
                            if data.len() > 0 {
                                match SymbolInstance::try_from(&mut data) {
                                    Ok(item) => {
                                        let start_instance = item.id; // update start instance
                                        return Ok(Some((
                                            item,
                                            State::HasData {
                                                ctx,
                                                start_instance,
                                                has_more,
                                                data,
                                            },
                                        )));
                                    }
                                    Err(e) => {
                                        //state = State::End;
                                        return Err(e);
                                    }
                                }
                            } else if has_more && all {
                                state = State::Request {
                                    ctx,
                                    start_instance,
                                };
                            } else {
                                state = State::End;
                            }
                        }
                        State::End => return Ok(None),
                    }
                }
            },
        )
    }
}

enum State<'a, T> {
    Request {
        ctx: &'a mut T,
        start_instance: u16,
    },
    HasData {
        ctx: &'a mut T,
        start_instance: u16,
        has_more: bool,
        data: Bytes,
    },
    End,
}

async fn get_attribute_list<'a, T: MessageService<Error = Error>>(
    ctx: &'a mut T,
    start_instance: u16,
) -> Result<(bool, Bytes)> {
    let resp = ctx
        .send(MessageRequest::new(
            0x55,
            EPath::default()
                .with_class(0x6B)
                .with_instance(start_instance),
            Bytes::from_static(&[
                02, 00, // number of attributes
                01, 00, // attribute 1 - symbol name
                02, 00, // attribute 2 - symbol type
            ]),
        ))
        .await?;
    if resp.reply_service != 0x55 + 0x80 {
        return Err(rseip_core::Error::<InnerError>::from_invalid_data()
            .with_context(format!(
                "unexpected reply service for write tag service: {:#0x}",
                resp.reply_service
            ))
            .into());
    }
    if !resp.status.is_ok() && !resp.status.has_more() {
        return Err(CipError::Cip(resp.status).into());
    }
    Ok((resp.status.has_more(), resp.data))
}

impl TryFrom<&mut Bytes> for SymbolInstance {
    type Error = Error;

    fn try_from(buf: &mut Bytes) -> Result<Self> {
        if buf.len() < 6 {
            return Err(rseip_core::Error::<InnerError>::from_invalid_data()
                .with_context("not enough data to decode")
                .into());
        }
        let id = buf.get_u16_le();
        buf.advance(2);
        let name_len = buf.get_u8() as usize;
        buf.advance(1);
        if buf.len() < name_len + 2 {
            return Err(rseip_core::Error::<InnerError>::from_invalid_data()
                .with_context("not enough data to decode")
                .into());
        }
        let name =
            String::from_utf8(buf.split_to(name_len).to_vec()).map_err(|e| e.utf8_error())?;
        let symbol_type = buf.get_u16_le();
        Ok(SymbolInstance {
            id,
            name,
            symbol_type,
        })
    }
}
