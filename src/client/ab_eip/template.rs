// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod decoder;

use super::{symbol::SymbolType, HasMore, CLASS_TEMPLATE, REPLY_MASK, SERVICE_TEMPLATE_READ};
use crate::{
    cip::{
        codec::LazyEncode,
        epath::EPath,
        service::{CommonServices, MessageService},
        CipError, MessageRequest,
    },
    error::invalid_data,
    Error, Result,
};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use decoder::{DefaultDefinitionDecoder, DefinitionDecoder};
use rseip_core::{String, StringExt};
use smallvec::SmallVec;
use std::{
    collections::HashMap,
    convert::TryFrom,
    mem,
    ops::{Deref, DerefMut},
    result::Result as StdResult,
};

#[async_trait::async_trait(?Send)]
pub trait TemplateService {
    /// fetch template instance for specified instance id
    async fn template_instance(&mut self, instance_id: u16) -> Result<Template>;
    /// read template definition
    fn read_template<'a>(
        &'a mut self,
        instance_id: u16,
        template: &Template,
    ) -> TemplateRead<'a, Self>
    where
        Self: Sized;
}

#[async_trait::async_trait(?Send)]
impl<T: MessageService<Error = Error>> TemplateService for T {
    /// fetch template instance for specified instance id
    async fn template_instance(&mut self, instance_id: u16) -> Result<Template> {
        let path = EPath::default()
            .with_class(CLASS_TEMPLATE)
            .with_instance(instance_id);
        let res = self.get_attribute_list(path, &[1, 2, 4, 5]).await?;
        Ok(res)
    }

    /// read template definition
    fn read_template<'a>(
        &'a mut self,
        instance_id: u16,
        template: &Template,
    ) -> TemplateRead<'a, Self>
    where
        Self: Sized,
    {
        let mut decoder: DefaultDefinitionDecoder = Default::default();
        decoder.member_count(template.member_count);
        TemplateRead {
            inner: self,
            instance_id,
            object_size: template.object_size * 4,
            member_count: template.member_count,
            offset: 0,
            read_size: 500,
            decoder,
        }
    }
}

pub struct TemplateRead<'a, T, D = DefaultDefinitionDecoder> {
    inner: &'a mut T,
    instance_id: u16,
    object_size: u32,
    member_count: u16,
    offset: u32,
    read_size: u16,
    decoder: D,
}

impl<'a, T, D: Default> TemplateRead<'a, T, D> {
    pub fn new(inner: &'a mut T) -> Self {
        Self {
            inner,
            instance_id: 0,
            object_size: 0,
            member_count: 0,
            offset: 0,
            read_size: 0,
            decoder: Default::default(),
        }
    }
}

impl<'a, T, D: DefinitionDecoder> TemplateRead<'a, T, D> {
    /// set template instance id
    pub fn instance_id(mut self, instance_id: u16) -> Self {
        self.instance_id = instance_id;
        self
    }

    /// number of bytes of the object
    pub fn object_size(mut self, object_size: u32) -> Self {
        self.object_size = object_size;
        self
    }

    /// number of member of the object
    pub fn member_count(mut self, member_count: u16) -> Self {
        self.member_count = member_count;
        self.decoder.member_count(member_count);
        self
    }

    /// offset to send request
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    /// number of bytes to read in a single request, as all the data may not fit in a single reply
    pub fn read_size(mut self, read_size: u16) -> Self {
        self.read_size = read_size;
        self
    }

    pub fn decoder<R: DefinitionDecoder>(self, mut decoder: R) -> TemplateRead<'a, T, R> {
        decoder.member_count(self.member_count);
        TemplateRead {
            inner: self.inner,
            instance_id: self.instance_id,
            object_size: self.object_size,
            member_count: self.member_count,
            offset: self.offset,
            read_size: self.read_size,
            decoder,
        }
    }

    pub fn decoder_mut(&mut self) -> &mut D {
        &mut self.decoder
    }
}

impl<'a, T, D> TemplateRead<'a, T, D>
where
    T: MessageService<Error = Error>,
    D: DefinitionDecoder,
    D::Error: Into<Error>,
{
    pub async fn call(mut self) -> Result<D::Item> {
        if self.member_count < 2 {
            return Err(invalid_data(
                "read template - need to initialize `member_count`",
            ));
        }
        if self.object_size < 23 {
            return Err(invalid_data(
                "read template - need to initialize `object_size`",
            ));
        }

        // Note: self.object_size = template.object_size * 4;
        // 23 bytes will not be included in the reply data.
        let total_bytes = self.object_size - 23;
        let mut offset = self.offset;
        while offset < total_bytes {
            // determine bytes to read
            let bytes_read = {
                // the initial offset should be 0;
                // after first fetch, offset = bytes received + 1
                let remaining = if offset == 0 {
                    total_bytes
                } else {
                    total_bytes - offset + 1
                };
                if remaining > self.read_size as u32 {
                    self.read_size
                } else {
                    remaining as u16
                }
            };

            let (has_more, data) =
                read_template(self.inner, self.instance_id, offset, bytes_read).await?;
            debug_assert!(data.len() > 0 && data.len() == bytes_read as usize);
            if offset == 0 {
                offset = data.len() as u32 + 1;
            } else {
                offset = offset + data.len() as u32;
            }

            // partially decode
            self.decoder.partial_decode(data).map_err(|e| e.into())?;
            if !has_more {
                // extract object
                let res = self.decoder.decode().map_err(|e| e.into())?;
                return Ok(res);
            }
        }
        Err(invalid_data("read template - offset out of range"))
    }
}

async fn read_template<'a, T>(
    ctx: &'a mut T,
    instance_id: u16,
    offset: u32,
    bytes_read: u16,
) -> Result<(bool, Bytes)>
where
    T: MessageService<Error = Error>,
{
    let path = EPath::default()
        .with_class(CLASS_TEMPLATE)
        .with_instance(instance_id);
    let data = LazyEncode {
        bytes_count: 6,
        f: |buf: &mut BytesMut| {
            buf.put_u32_le(offset);
            buf.put_u16_le(bytes_read);
            Ok(())
        },
    };
    let req = MessageRequest::new(SERVICE_TEMPLATE_READ, path, data);
    let resp = ctx.send(req).await?;
    if resp.reply_service != SERVICE_TEMPLATE_READ + REPLY_MASK {
        return Err(invalid_data(format!(
            "read template - unexpected reply service: {:#0x}",
            resp.reply_service
        )));
    }
    if !resp.status.is_ok() && !resp.status.has_more() {
        return Err(CipError::Cip(resp.status).into());
    }
    Ok((resp.has_more(), resp.data))
}

/// template definition
#[derive(Debug, Default)]
pub struct TemplateDefinition {
    /// template name
    pub(crate) name: String,
    /// template members
    pub(crate) members: HashMap<String, MemberInfo>,
}

impl TemplateDefinition {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Deref for TemplateDefinition {
    type Target = HashMap<String, MemberInfo>;
    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl DerefMut for TemplateDefinition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.members
    }
}

/// template member definition
#[derive(Debug, Clone)]
pub struct MemberInfo {
    /// member name
    pub name: String,

    /// array_size = 0 if atomic type;
    ///
    /// array_size > 0 if array type;
    ///
    /// array_size is bit location if boolean type
    /// - range 0-31
    /// - range 0-7 if mapped to a SINT
    pub array_size: u16,
    /// member type info
    pub type_info: SymbolType,
    /// offset position of data bytes
    pub offset: u32,
}

/// template object
#[derive(Debug, Clone)]
pub struct Template {
    /// structure handle, Tag Type Parameter used in Read/Write Tag service
    pub handle: u16,
    /// template member count
    pub member_count: u16,
    /// template object definition size, number of 32-bit words
    pub object_size: u32,
    /// template structure size, number of bytes of structure data to transfer in Read/Write Tag service
    pub struct_size: u32,
}

impl TryFrom<Bytes> for Template {
    type Error = Error;
    fn try_from(mut buf: Bytes) -> Result<Self> {
        if buf.len() < 4 {
            return Err(invalid_data("template - not enough data to decode"));
        }
        let count = buf.get_u16_le();
        if count != 4 {
            return Err(invalid_data(
                "template - unexpected count of items returned",
            ));
        }
        if buf.len() < 28 {
            return Err(invalid_data("template - not enough data to decode"));
        }

        let handle = decode_attr(&mut buf, 1, |buf| Ok(buf.get_u16_le()))?;
        let member_count = decode_attr(&mut buf, 2, |buf| Ok(buf.get_u16_le()))?;
        let object_size = decode_attr(&mut buf, 4, |buf| Ok(buf.get_u32_le()))?;
        let struct_size = decode_attr(&mut buf, 5, |buf| Ok(buf.get_u32_le()))?;

        if buf.len() != 0 {
            return Err(invalid_data("template - too much data to decode"));
        }
        Ok(Self {
            handle,
            member_count,
            object_size,
            struct_size,
        })
    }
}

fn decode_attr<F, R>(buf: &mut Bytes, attr_id: u16, f: F) -> Result<R>
where
    F: Fn(&mut Bytes) -> Result<R>,
{
    let id = buf.get_u16_le();
    let status = buf.get_u16_le();
    if status != 0 {
        return Err(invalid_data(format!(
            "attribute - bad attribute[{}] status: {:#0x}",
            id, status
        )));
    }
    if attr_id != id {
        return Err(invalid_data(format!(
            "attribute -unexpected attribute[{}]",
            id
        )));
    }
    f(buf)
}
