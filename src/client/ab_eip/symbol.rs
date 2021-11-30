use crate::error::{invalid_data, InnerError};
use crate::{Error, Result};
use bytes::Buf;
use bytes::Bytes;
use core::fmt;
use futures_util::stream;
use futures_util::Stream;
use rseip_cip::{epath::EPath, service::MessageService, CipError, MessageRequest};
use rseip_core::hex::AsHex;
use std::convert::TryFrom;

use super::HasMore;

/// symbol instance
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct SymbolInstance {
    /// instance id
    pub id: u16,
    /// symbol name
    pub name: String,
    /// symbol data type
    pub symbol_type: SymbolType,
}

impl fmt::Debug for SymbolInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SymbolInstance")
            .field("id", &self.id.as_hex())
            .field("name", &self.name)
            .field("symbol_type", &self.symbol_type)
            .finish()
    }
}

impl SymbolInstance {
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
        self.0 = self.0 | instance_id;
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
        self.0 = self.0 | ((pos as u16) << 8);
        self
    }

    pub fn finish(self) -> SymbolType {
        SymbolType(self.0)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
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
        return Err(invalid_data(format!(
            "unexpected reply service for write tag service: {:#0x}",
            resp.reply_service
        )));
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
            return Err(invalid_data("not enough data to decode"));
        }
        let name =
            String::from_utf8(buf.split_to(name_len).to_vec()).map_err(|e| e.utf8_error())?;
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
