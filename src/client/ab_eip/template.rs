use super::{symbol::SymbolType, HasMore};
use crate::{error::invalid_data, Error, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use rseip_cip::{
    codec::LazyEncode,
    epath::EPath,
    service::{CommonServices, MessageService},
    CipError, MessageRequest,
};
use rseip_core::{String, StringExt};
use smallvec::SmallVec;
use std::{collections::HashMap, convert::TryFrom, mem, result::Result as StdResult};

const SERVICE_TEMPLATE_READ: u8 = 0x4C;
const REPLY_TEMPLATE_READ: u8 = 0x4C + 0x80;
const CLASS_TEMPLATE: u16 = 0x6C;

/// template definition decoder
pub trait DefinitionDecoder {
    type Item;
    type Error;

    /// set member count;
    /// to decode the definition, it need to specify the number of members.
    fn member_count(&mut self, member_count: u16);

    /// partial decode
    fn partial_decode(&mut self, buf: Bytes) -> StdResult<(), Self::Error>;

    /// finally decode
    fn decode(&mut self) -> StdResult<Self::Item, Self::Error>;
}

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
    if resp.reply_service != REPLY_TEMPLATE_READ {
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
    pub name: String,
    /// template members
    pub members: HashMap<String, MemberInfo>,
}

/// default template definition decoder
#[derive(Debug, Default)]
pub struct DefaultDefinitionDecoder {
    /// template name
    name: String,
    /// members of template
    members: SmallVec<[MemberInfo; 8]>,
    /// the exact number of members
    member_count: u16,
    /// index to track when decode member names
    index: u16,
}

impl DefinitionDecoder for DefaultDefinitionDecoder {
    type Error = Error;
    type Item = TemplateDefinition;

    fn member_count(&mut self, member_count: u16) {
        self.member_count = member_count;
    }

    fn partial_decode(&mut self, mut buf: Bytes) -> StdResult<(), Self::Error> {
        if self.member_count < 2 {
            return Err(invalid_data(
                "template definition - need to initialize `member_count`",
            ));
        }
        while self.members.len() < self.member_count as usize {
            //TODO: validate buf.len()
            let item = MemberInfo {
                name: Default::default(),
                array_size: buf.get_u16_le(),
                type_info: SymbolType(buf.get_u16_le()),
                offset: buf.get_u32_le(),
            };
            self.members.push(item);
        }
        let mut strings = buf.split(|v| *v == 0);
        if self.name.is_empty() {
            if let Some(buf) = strings.next() {
                //TODO: improve it
                let mut parts = buf.split(|v| *v == 0x3B);
                let buf = parts.next().unwrap();
                self.name = String::from_utf8_lossy(buf).into_owned().into();
            }
        }
        while let Some(buf) = strings.next() {
            if self.index < self.member_count {
                self.members[self.index as usize].name =
                    String::from_utf8_lossy(buf).into_owned().into();
                self.index += 1;
            } else {
                break;
            }
        }

        Ok(())
    }

    /// finally decode, return the target object and reset inner state of the decoder
    fn decode(&mut self) -> StdResult<Self::Item, Self::Error> {
        if self.member_count < 2 {
            return Err(invalid_data(
                "template definition - need to initialize `member_count`",
            ));
        }
        if self.index < self.member_count {
            return Err(invalid_data(
                "template definition - not enough data to decode",
            ));
        }
        self.index = 0;
        self.member_count = 0;
        let map: HashMap<_, _> = self
            .members
            .drain(..)
            .map(|item| (item.name.clone(), item))
            .collect();
        Ok(TemplateDefinition {
            name: mem::take(&mut self.name),
            members: map,
        })
    }
}

/// template member definition
#[derive(Debug, Clone)]
pub struct MemberInfo {
    /// member name
    pub name: String,
    /// array_size > 0 if array
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
