// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::{interceptor::HasMoreInterceptor, HasMore, CLASS_SYMBOL, REPLY_MASK};
use crate::{
    cip::{epath::EPath, service::MessageService, MessageRequest},
    ClientError, Result,
};
use bytes::{Buf, Bytes};
use core::{convert::TryFrom, fmt, slice, str};
use futures_util::{stream, Stream};
use rseip_cip::MessageReplyInterface;
use rseip_core::{codec::BytesHolder, hex::AsHex, Error};
use std::borrow::Cow;

/// symbol instance
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct SymbolInstance<'a> {
    /// instance id
    pub id: u16,
    /// symbol name
    pub name: Cow<'a, str>,
    /// symbol data type
    pub symbol_type: SymbolType,
}

impl fmt::Debug for SymbolInstance<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SymbolInstance")
            .field("id", &self.id.as_hex())
            .field("name", &self.name)
            .field("symbol_type", &self.symbol_type)
            .finish()
    }
}

impl SymbolInstance<'_> {
    /// symbol name that contains `:`
    #[inline]
    pub fn is_module_defined(&self) -> bool {
        self.name.contains(':')
    }
}

#[derive(Debug, Default)]
pub struct SymbolTypeBuilder(u16);
impl SymbolTypeBuilder {
    /// panic if instance_id > 0xFFF
    pub fn structure(mut self, instance_id: u16) -> Self {
        if instance_id > 0xFFF {
            panic!("instance id out of range");
        }
        // set bit 15 =1
        const MASK: u16 = 0b0111 << 12;
        self.0 = (self.0 & MASK) | (1 << 15);
        // set instance id
        self.0 |= instance_id;
        self
    }

    pub fn atomic(mut self, type_code: u8) -> Self {
        const MASK: u16 = 0b0111 << 12;
        self.0 = (self.0 & MASK) | (type_code as u16);
        self
    }

    /// panics if dims >= 4
    pub fn dims(mut self, dims: u8) -> Self {
        if dims >= 4 {
            panic!("dims out of range");
        }
        const MASK: u16 = 0b11 << 13;
        self.0 = (self.0 & (!MASK)) | ((dims as u16) << 13);
        self
    }

    /// panics if:
    ///
    /// - pos > 7
    /// - type is not bool
    pub fn bit_pos(mut self, pos: u8) -> Self {
        if pos > 7 {
            panic!("pos out of range")
        }
        if !SymbolType(self.0).is_bool() {
            panic!("type not bool")
        }
        self.0 |= (pos as u16) << 8;
        self
    }

    pub fn finish(self) -> SymbolType {
        SymbolType(self.0)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct SymbolType(pub(crate) u16);

impl SymbolType {
    pub fn builder() -> SymbolTypeBuilder {
        Default::default()
    }

    /// is struct
    #[inline]
    pub fn is_struct(&self) -> bool {
        const MASK: u16 = 1 << 15;
        self.0 & MASK == MASK
    }

    /// is atomic
    #[inline]
    pub fn is_atomic(&self) -> bool {
        !self.is_struct()
    }

    /// system predefined struct
    #[inline]
    pub fn is_predefined(&self) -> bool {
        self.instance_id().map(|v| v > 0xEFF).unwrap_or_default()
    }

    /// type code if atomic; range from 0x01-0xFF
    #[inline]
    pub fn type_code(&self) -> Option<u8> {
        if !self.is_struct() {
            Some((self.0 & 0xFF) as u8)
        } else {
            None
        }
    }

    /// dims: 0, 1, 2, 3
    #[inline]
    pub fn dims(&self) -> u8 {
        ((self.0 >> 13) as u8) & 0b11
    }

    /// bool type?
    #[inline]
    pub fn is_bool(&self) -> bool {
        if !self.is_struct() {
            let v = self.0 & 0xFF;
            v == 0xC1
        } else {
            false
        }
    }

    /// only if it's bool; bit position: 0-7
    #[inline]
    pub fn bit_pos(&self) -> Option<u8> {
        if self.is_bool() {
            let v = (self.0 >> 8) & 0b111;
            Some(v as u8)
        } else {
            None
        }
    }

    /// template instance id if struct; range from 0x100-0xFFF
    #[inline]
    pub fn instance_id(&self) -> Option<u16> {
        if self.is_struct() {
            Some(self.0 & 0xFFF)
        } else {
            None
        }
    }
}

impl fmt::Debug for SymbolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("SymbolType");
        if self.is_struct() {
            d.field("type", &"struct");
        } else {
            d.field("type", &"atomic");
        }
        d.field("dims", &self.dims());
        self.instance_id()
            .map(|v| d.field("instance_id", &v.as_hex()));
        self.type_code().map(|v| d.field("type_code", &v.as_hex()));
        self.bit_pos().map(|v| d.field("bit_pos", &v));
        d.finish()
    }
}

impl From<SymbolType> for u16 {
    fn from(src: SymbolType) -> Self {
        src.0
    }
}

/// only instances created are returned.
/// Any symbol instances that represents tags whose External Access is set to None are not included in the reply data.
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
    ///
    /// default true
    pub fn retrieve_all(mut self, all: bool) -> Self {
        self.all = all;
        self
    }
}

impl<'a, T: MessageService<Error = ClientError>> GetInstanceAttributeList<'a, T> {
    pub fn call(self) -> impl Stream<Item = Result<SymbolInstance<'a>>> {
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
                            match get_attribute_list(ctx, start_instance).await {
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
                            if !data.is_empty() {
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
                                        //dbg!(&e);
                                        //state = State::End;
                                        return Err(e);
                                    }
                                }
                            } else if has_more && all {
                                //dbg!(has_more, "new request");
                                state = State::Request {
                                    ctx,
                                    start_instance: start_instance + 1,
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

async fn get_attribute_list<T: MessageService<Error = ClientError>>(
    ctx: &mut T,
    start_instance: u16,
) -> Result<(bool, Bytes)> {
    const SERVICE_GET_INSTANCE_ATTRIBUTE_LIST: u8 = 0x55;
    let path = EPath::default()
        .with_class(CLASS_SYMBOL)
        .with_instance(start_instance);
    let data: &[u8] = &[
        0x02, 0x00, // number of attributes
        0x01, 0x00, // attribute 1 - symbol name
        0x02, 0x00, // attribute 2 - symbol type
    ];
    let resp: HasMoreInterceptor<BytesHolder> = ctx
        .send(MessageRequest::new(
            SERVICE_GET_INSTANCE_ATTRIBUTE_LIST,
            path,
            data,
        ))
        .await?;
    resp.expect_service::<ClientError>(SERVICE_GET_INSTANCE_ATTRIBUTE_LIST + REPLY_MASK)?;
    Ok((resp.0.status.has_more(), resp.0.data.into()))
}

impl TryFrom<&mut Bytes> for SymbolInstance<'_> {
    type Error = ClientError;

    fn try_from(buf: &mut Bytes) -> Result<Self> {
        if buf.remaining() < 6 {
            return Err(Error::invalid_length(buf.len(), 6));
        }
        let id = buf.get_u16_le();
        buf.advance(2);
        let name_len = buf.get_u8() as usize;
        buf.advance(1);
        if buf.remaining() < name_len + 2 {
            return Err(Error::invalid_length(buf.remaining(), name_len + 2));
        }
        let name = unsafe {
            let name_buf = buf.split_to(name_len);
            let buf = name_buf.as_ptr();
            let buf = slice::from_raw_parts(buf, name_len as usize);
            let name = str::from_utf8_unchecked(buf);
            Cow::from(name)
        };
        let symbol_type = buf.get_u16_le();
        Ok(SymbolInstance {
            id,
            name,
            symbol_type: SymbolType(symbol_type),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_type() {
        let sym_type = SymbolType(0x82E9);
        assert!(sym_type.is_struct());
        assert!(!sym_type.is_atomic());
    }
}
