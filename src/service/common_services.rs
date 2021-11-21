mod multiple_packet;

use super::*;
use super::{reply::*, request::*};
use crate::{
    cip::{epath::EPath, MessageRouterRequest},
    codec::{encode::LazyEncode, Encodable},
    eip::EipError,
    Error, Result,
};
use bytes::{Buf, BufMut, Bytes, BytesMut};
pub use multiple_packet::MultipleServicePacket;
use std::convert::TryFrom;

/// common services
#[async_trait::async_trait(?Send)]
pub trait CommonServices: MessageRouter {
    /// invoke the Get_Attribute_All service
    async fn get_attribute_all<R>(&mut self, path: EPath) -> Result<R>
    where
        R: TryFrom<Bytes>,
        R::Error: Into<crate::Error> + std::error::Error,
    {
        let mr = MessageRouterRequest {
            service_code: 0x01,
            path,
            data: (),
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        R::try_from(reply.data).map_err(|e| e.into())
    }

    /// invoke the Set_Attribute_All service
    async fn set_attribute_all<D: Encodable>(&mut self, path: EPath, attrs: D) -> Result<()> {
        let mr = MessageRouterRequest {
            service_code: 0x02,
            path,
            data: attrs,
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        Ok(())
    }

    /// invoke the Get_Attribute_List
    async fn get_attribute_list(
        &mut self,
        path: EPath,
        attrs: Vec<GetAttributeRequestItem>,
    ) -> Result<Vec<AttributeReply>> {
        let attrs_len = attrs.len();
        assert!(attrs_len <= u16::MAX as usize);
        let mr = MessageRouterRequest {
            service_code: 0x03,
            path,
            data: LazyEncode {
                f: |buf: &mut BytesMut| {
                    buf.put_u16_le(attrs_len as u16);
                    for item in attrs.iter() {
                        buf.put_u16_le(item.id);
                    }
                    Ok(())
                },
                bytes_count: 2 + attrs_len * 2,
            },
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }

        decode_get_attr_list(reply.data, &attrs)
    }

    /// invoke the Set_Attribute_List service
    async fn set_attribute_list<D: Encodable, R: Encodable>(
        &mut self,
        path: EPath,
        attrs: Vec<SetAttributeRequestItem<D>>,
    ) -> Result<Vec<AttributeReply>> {
        let attrs_len = attrs.len();
        assert!(attrs_len <= u16::MAX as usize);
        let mr = MessageRouterRequest {
            service_code: 0x04,
            path,
            data: LazyEncode {
                f: |buf: &mut BytesMut| {
                    buf.put_u16_le(attrs_len as u16);
                    for item in attrs.iter() {
                        buf.put_u16_le(item.id);
                    }
                    Ok(())
                },
                bytes_count: 2 + attrs_len * 2,
            },
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        decode_set_attr_list(reply.data, &attrs)
    }

    /// invoke the Reset service
    async fn reset(&mut self, path: EPath) -> Result<()> {
        let mr = MessageRouterRequest {
            service_code: 0x05,
            path,
            data: (),
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        Ok(())
    }

    /// invoke the Start service
    async fn start(&mut self, path: EPath) -> Result<()> {
        let mr = MessageRouterRequest {
            service_code: 0x06,
            path,
            data: (),
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        Ok(())
    }

    /// invoke the Stop service
    async fn stop(&mut self, path: EPath) -> Result<()> {
        let mr = MessageRouterRequest {
            service_code: 0x07,
            path,
            data: (),
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        Ok(())
    }

    /// invoke the Create service
    async fn create<D: Encodable>(&mut self, path: EPath, data: D) -> Result<()> {
        let mr = MessageRouterRequest {
            service_code: 0x08,
            path,
            data,
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        Ok(())
    }

    /// invoke the Delete service
    async fn delete(&mut self, path: EPath) -> Result<()> {
        let mr = MessageRouterRequest {
            service_code: 0x09,
            path,
            data: (),
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        Ok(())
    }

    /// invoke the Apply_Attributes service
    async fn apply_attributes<D, R>(&mut self, path: EPath, data: D) -> Result<R>
    where
        D: Encodable,
        R: TryFrom<Bytes>,
        R::Error: Into<crate::Error> + std::error::Error,
    {
        let mr = MessageRouterRequest {
            service_code: 0x0D,
            path,
            data,
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        R::try_from(reply.data).map_err(|e| e.into())
    }

    /// invoke the Get_Attribute_Single service
    async fn get_attribute_single<R>(&mut self, path: EPath) -> Result<R>
    where
        R: TryFrom<Bytes>,
        R::Error: Into<crate::Error> + std::error::Error,
    {
        let mr = MessageRouterRequest {
            service_code: 0x0E,
            path,
            data: (),
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        R::try_from(reply.data).map_err(|e| e.into())
    }

    /// invoke the Set_Attribute_Single service
    async fn set_attribute_single<D: Encodable>(&mut self, path: EPath, data: D) -> Result<()> {
        let mr = MessageRouterRequest {
            service_code: 0x10,
            path,
            data,
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        Ok(())
    }

    /// invoke the Restore service
    async fn restore(&mut self, path: EPath) -> Result<()> {
        let mr = MessageRouterRequest {
            service_code: 0x15,
            path,
            data: (),
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        Ok(())
    }

    /// invoke the Save service
    async fn save(&mut self, path: EPath) -> Result<()> {
        let mr = MessageRouterRequest {
            service_code: 0x16,
            path,
            data: (),
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        Ok(())
    }

    /// invoke the Nop service
    async fn no_operation(&mut self, path: EPath) -> Result<()> {
        let mr = MessageRouterRequest {
            service_code: 0x17,
            path,
            data: (),
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        Ok(())
    }

    /// invoke the Get_Member service
    async fn get_member<R>(&mut self, path: EPath) -> Result<R>
    where
        R: TryFrom<Bytes>,
        R::Error: Into<crate::Error> + std::error::Error,
    {
        let mr = MessageRouterRequest {
            service_code: 0x18,
            path,
            data: (),
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        R::try_from(reply.data).map_err(|e| e.into())
    }

    /// invoke the Set_Member service
    async fn set_member<D: Encodable>(&mut self, path: EPath, data: D) -> Result<()> {
        let mr = MessageRouterRequest {
            service_code: 0x19,
            path,
            data,
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        Ok(())
    }

    /// invoke the Insert_Member service
    async fn insert_member<D: Encodable>(&mut self, path: EPath, data: D) -> Result<()> {
        let mr = MessageRouterRequest {
            service_code: 0x1A,
            path,
            data,
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        Ok(())
    }

    /// invoke the Remove_Member service
    async fn remove_member(&mut self, path: EPath) -> Result<()> {
        let mr = MessageRouterRequest {
            service_code: 0x1B,
            path,
            data: (),
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        Ok(())
    }

    /// invoke the Group_Sync service
    async fn group_sync(&mut self, path: EPath) -> Result<()> {
        let mr = MessageRouterRequest {
            service_code: 0x1B,
            path,
            data: (),
        };
        let reply = self.send(mr).await?;
        if !reply.status.is_ok() {
            return Err(Error::MessageRequestError(reply));
        }
        Ok(())
    }

    /// multiple service packet
    #[inline]
    fn multiple_service(&mut self) -> MultipleServicePacket<'_, Self>
    where
        Self: Sized,
    {
        MultipleServicePacket::new(self)
    }
}

#[async_trait::async_trait(?Send)]
impl<T: MessageRouter> CommonServices for T {}

fn decode_get_attr_list(
    mut buf: Bytes,
    attrs: &Vec<GetAttributeRequestItem>,
) -> Result<Vec<AttributeReply>> {
    if buf.len() < 2 {
        return Err(Error::Eip(EipError::InvalidData));
    }
    let count = buf.get_u16_le() as usize;
    if count != attrs.len() {
        return Err(Error::Eip(EipError::InvalidData));
    }
    let mut results = vec![];
    for attr in attrs {
        if buf.len() < 4 + attr.size as usize {
            return Err(Error::Eip(EipError::InvalidData));
        }
        let id = buf.get_u16_le();
        let status = buf.get_u16_le();
        if id != attr.id {
            return Err(Error::Eip(EipError::InvalidData));
        }

        results.push(AttributeReply {
            id,
            status,
            data: if status == 0x00 {
                buf.split_to(attr.size as usize)
            } else {
                Bytes::default()
            },
        })
    }

    if buf.len() != 0 {
        return Err(Error::Eip(EipError::InvalidData));
    }

    Ok(results)
}

fn decode_set_attr_list<T>(
    mut buf: Bytes,
    attrs: &Vec<SetAttributeRequestItem<T>>,
) -> Result<Vec<AttributeReply>> {
    if buf.len() < 2 {
        return Err(Error::Eip(EipError::InvalidData));
    }
    let count = buf.get_u16_le() as usize;
    if count != attrs.len() {
        return Err(Error::Eip(EipError::InvalidData));
    }
    let mut results = vec![];
    for attr in attrs {
        if buf.len() < 4 + attr.size as usize {
            return Err(Error::Eip(EipError::InvalidData));
        }
        let id = buf.get_u16_le();
        let status = buf.get_u16_le();
        if id != attr.id {
            return Err(Error::Eip(EipError::InvalidData));
        }

        results.push(AttributeReply {
            id,
            status,
            data: if status == 0x00 {
                buf.split_to(attr.size as usize)
            } else {
                Bytes::default()
            },
        })
    }

    if buf.len() != 0 {
        return Err(Error::Eip(EipError::InvalidData));
    }

    Ok(results)
}
