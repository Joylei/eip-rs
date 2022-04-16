// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::{
    interceptor::HasMoreInterceptor, symbol::SymbolType, HasMore, CLASS_TEMPLATE, REPLY_MASK,
    SERVICE_TEMPLATE_READ,
};
use crate::{
    cip::{
        epath::EPath,
        service::{CommonServices, MessageService},
        MessageRequest,
    },
    ClientError,
};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::{
    ops::{Deref, DerefMut},
    str,
};
use rseip_cip::MessageReplyInterface;
use rseip_core::{
    codec::{BytesHolder, Decode, Decoder},
    Error,
};
use smallvec::SmallVec;
use std::collections::HashMap;

#[async_trait::async_trait]
pub trait AbTemplateService {
    /// fetch template instance for specified instance id
    async fn find_template(&mut self, instance_id: u16) -> Result<Template, ClientError>;

    /// read template definition
    fn read_template<'a>(&'a mut self, template: &Template) -> TemplateRead<'a, Self>
    where
        Self: Sized;
}

#[async_trait::async_trait]
impl<T: MessageService<Error = ClientError>> AbTemplateService for T {
    /// fetch template instance for specified instance id
    async fn find_template(&mut self, instance_id: u16) -> Result<Template, ClientError> {
        let path = EPath::default()
            .with_class(CLASS_TEMPLATE)
            .with_instance(instance_id);
        let mut res: Template = self.get_attribute_list(path, &[1, 2, 4, 5]).await?;
        res.instance_id = instance_id;
        Ok(res)
    }

    /// read template definition
    fn read_template<'a>(&'a mut self, template: &Template) -> TemplateRead<'a, Self>
    where
        Self: Sized,
    {
        TemplateRead {
            inner: self,
            instance_id: template.instance_id,
            object_size: template.object_size,
            member_count: template.member_count,
            buf: Default::default(),
        }
    }
}

pub struct TemplateRead<'a, T> {
    inner: &'a mut T,
    instance_id: u16,
    object_size: u32,
    member_count: u16,
    buf: BytesMut,
}

impl<'a, T> TemplateRead<'a, T> {
    pub fn new(inner: &'a mut T) -> Self {
        Self {
            inner,
            instance_id: 0,
            object_size: 0,
            member_count: 0,
            buf: Default::default(),
        }
    }
}

impl<'a, T> TemplateRead<'a, T> {
    /// template instance id
    pub fn instance_id(mut self, instance_id: u16) -> Self {
        self.instance_id = instance_id;
        self
    }

    /// template object definition size
    pub fn object_size(mut self, object_size: u32) -> Self {
        self.object_size = object_size;
        self
    }

    /// number of members
    pub fn member_count(mut self, member_count: u16) -> Self {
        self.member_count = member_count;
        self
    }
}

impl<'a, T> TemplateRead<'a, T>
where
    T: MessageService<Error = ClientError>,
{
    pub async fn call<'de>(&'de mut self) -> Result<TemplateDefinition<'de>, ClientError> {
        const HEADER_SIZE: u32 = 23;
        if self.member_count == 0 {
            return Err(Error::custom(
                "read template - need to initialize `member_count`",
            ));
        }

        let total_bytes = {
            let object_size = self.object_size * 4;
            if object_size <= HEADER_SIZE {
                return Err(Error::custom(
                    "read template - need to initialize `object_size`",
                ));
            }
            // header(23 bytes) will not be included in the reply data.
            let total_bytes = object_size - HEADER_SIZE;
            // calc padding
            let v = total_bytes % 4;
            if v == 0 {
                total_bytes
            } else {
                total_bytes - v + 4
            }
        };

        self.buf.clear();

        // the initial offset should be 0;
        // after first fetch, offset = bytes received + 1
        let mut buf_len = 0;
        while buf_len < total_bytes {
            let (offset, remaining) = if buf_len == 0 {
                (0, total_bytes)
            } else {
                (buf_len, total_bytes - buf_len)
            };
            //dbg!(total_bytes, offset);
            let (has_more, data) = read_template(
                self.inner,
                self.instance_id,
                offset as u32,
                remaining as u16,
            )
            .await?;
            debug_assert!(!data.is_empty() && data.len() as u32 <= remaining);
            self.buf.put_slice(&data[..]);
            //dbg!(data.len(), self.buf.len());
            if !has_more {
                // extract object
                let res = decode_definition(&mut self.buf, self.member_count)?;
                return Ok(res);
            }
            buf_len = self.buf.len() as u32;
        }
        Err(Error::custom("read template - offset out of range"))
    }
}

async fn read_template<T>(
    ctx: &mut T,
    instance_id: u16,
    offset: u32,
    bytes_read: u16,
) -> Result<(bool, Bytes), ClientError>
where
    T: MessageService<Error = ClientError>,
{
    let path = EPath::default()
        .with_class(CLASS_TEMPLATE)
        .with_instance(instance_id);
    let data = (offset, bytes_read);
    let req = MessageRequest::new(SERVICE_TEMPLATE_READ, path, data);
    let resp: HasMoreInterceptor<BytesHolder> = ctx.send(req).await?;
    resp.expect_service::<ClientError>(SERVICE_TEMPLATE_READ + REPLY_MASK)?;
    Ok((resp.0.has_more(), resp.0.data.into()))
}

/// template definition
#[derive(Debug, Default)]
pub struct TemplateDefinition<'a> {
    /// template name
    pub(crate) name: &'a str,
    /// template members
    pub(crate) members: HashMap<&'a str, MemberInfo<'a>>,
}

impl TemplateDefinition<'_> {
    pub fn name(&self) -> &str {
        self.name
    }
}

impl<'a> Deref for TemplateDefinition<'a> {
    type Target = HashMap<&'a str, MemberInfo<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl<'a> DerefMut for TemplateDefinition<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.members
    }
}

/// template member definition
#[derive(Debug, Clone, Default)]
pub struct MemberInfo<'de> {
    /// member name
    pub name: &'de str,

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
    pub instance_id: u16,
    /// structure handle, Tag Type Parameter used in Read/Write Tag service
    pub handle: u16,
    /// template member count
    pub member_count: u16,
    /// template object definition size, number of 32-bit words
    pub object_size: u32,
    /// template structure size, number of bytes of structure data to transfer in Read/Write Tag service
    pub struct_size: u32,
}

// -- decode --

impl<'de> Decode<'de> for Template {
    fn decode<D>(mut decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.ensure_size(28)?;
        let count = decoder.decode_u16(); // buf[0..2]
        if count != 4 {
            return Err(Error::custom(
                "template - unexpected count of items returned",
            ));
        }

        let handle: u16 = decode_attr(&mut decoder, 1)?;
        let member_count: u16 = decode_attr(&mut decoder, 2)?;
        let object_size: u32 = decode_attr(&mut decoder, 4)?;
        let struct_size: u32 = decode_attr(&mut decoder, 5)?;

        if decoder.buf().has_remaining() {
            return Err(Error::custom("template - too much data to decode"));
        }
        Ok(Self {
            instance_id: 0,
            handle,
            member_count,
            object_size,
            struct_size,
        })
    }
}

fn decode_attr<'de, D, R>(buf: &mut D, attr_id: u16) -> Result<R, D::Error>
where
    D: Decoder<'de>,
    R: Decode<'de>,
{
    let id = buf.decode_u16();
    let status = buf.decode_u16();
    if status != 0 {
        return Err(Error::custom(format!(
            "attribute - bad attribute[{}] status: {:#0x}",
            id, status
        )));
    }
    if attr_id != id {
        return Err(Error::custom(format!(
            "attribute -unexpected attribute[{}]",
            id
        )));
    }
    buf.decode_any()
}

fn decode_definition<'de>(
    buf: &'de mut BytesMut,
    member_count: u16,
) -> Result<TemplateDefinition<'de>, ClientError> {
    let mut members: SmallVec<[MemberInfo<'_>; 8]> = SmallVec::with_capacity(member_count as usize);
    for _ in 0..member_count {
        let item = MemberInfo {
            name: Default::default(),
            array_size: buf.get_u16_le(),
            type_info: SymbolType(buf.get_u16_le()),
            offset: buf.get_u32_le(),
        };
        members.push(item);
    }
    let mut strings = (&buf[..]).split(|v| *v == 0).map(decode_name);
    let mut get_name = || {
        strings.next().ok_or_else(|| {
            ClientError::custom("read template - unexpected eof while decoding names")
        })
    };
    let name = get_name()?;
    for index in 0..member_count {
        let name = get_name()?;
        members[index as usize].name = name;
    }
    Ok(TemplateDefinition {
        name,
        members: members.drain(..).map(|item| (item.name, item)).collect(),
    })
}

/// name might contains `;`,  truncate to get the name
#[inline]
fn decode_name(buf: &[u8]) -> &str {
    // split by semi-colon
    let mut parts = buf.split(|v| *v == 0x3B);
    let name_buf = parts.next().unwrap();
    unsafe { str::from_utf8_unchecked(name_buf) }
}
