// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod multiple_packet;

use super::*;
use crate::{epath::EPath, error::cip_error_status, *};
pub use multiple_packet::MultipleServicePacket;
use rseip_core::codec::{Decode, Encode, SliceContainer};

/// common services
#[async_trait::async_trait(?Send)]
pub trait CommonServices: MessageService {
    /// invoke the Get_Attribute_All service
    async fn get_attribute_all<'de, R>(&mut self, path: EPath) -> StdResult<R, Self::Error>
    where
        R: Decode<'de> + 'static,
    {
        let req = MessageRequest {
            service_code: 0x01,
            path,
            data: (),
        };
        let reply: MessageReply<R> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(reply.data)
    }

    /// invoke the Set_Attribute_All service
    async fn set_attribute_all<D: Encode>(
        &mut self,
        path: EPath,
        attrs: D,
    ) -> StdResult<(), Self::Error> {
        let req = MessageRequest {
            service_code: 0x02,
            path,
            data: attrs,
        };
        let reply: MessageReply<()> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(())
    }

    /// invoke the Get_Attribute_List
    async fn get_attribute_list<'de, R>(
        &mut self,
        path: EPath,
        attrs: &[u16],
    ) -> StdResult<R, Self::Error>
    where
        R: Decode<'de> + 'static,
    {
        let attrs_len = attrs.len();
        assert!(attrs_len <= u16::MAX as usize);
        let req = MessageRequest {
            service_code: 0x03,
            path,
            data: (
                attrs_len as u16,
                SliceContainer::new(attrs).with_bytes_count(2 * attrs_len),
            ),
        };
        let reply: MessageReply<R> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }

        Ok(reply.data)
    }

    /// invoke the Set_Attribute_List service
    async fn set_attribute_list<'de, D, R>(
        &mut self,
        path: EPath,
        attrs: D,
    ) -> StdResult<R, Self::Error>
    where
        D: Encode,
        R: Decode<'de> + 'static,
    {
        let req = MessageRequest {
            service_code: 0x04,
            path,
            data: attrs,
        };
        let reply: MessageReply<R> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(reply.data)
    }

    /// invoke the Reset service
    async fn reset(&mut self, path: EPath) -> StdResult<(), Self::Error> {
        let req = MessageRequest {
            service_code: 0x05,
            path,
            data: (),
        };
        let reply: MessageReply<()> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(())
    }

    /// invoke the Start service
    async fn start(&mut self, path: EPath) -> StdResult<(), Self::Error> {
        let req = MessageRequest {
            service_code: 0x06,
            path,
            data: (),
        };
        let reply: MessageReply<()> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(())
    }

    /// invoke the Stop service
    async fn stop(&mut self, path: EPath) -> StdResult<(), Self::Error> {
        let req = MessageRequest {
            service_code: 0x07,
            path,
            data: (),
        };
        let reply: MessageReply<()> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(())
    }

    /// invoke the Create service
    async fn create<'de, D, R>(&mut self, path: EPath, data: D) -> StdResult<R, Self::Error>
    where
        D: Encode,
        R: Decode<'de> + 'static,
    {
        let req = MessageRequest {
            service_code: 0x08,
            path,
            data,
        };
        let reply: MessageReply<R> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(reply.data)
    }

    /// invoke the Delete service
    async fn delete(&mut self, path: EPath) -> StdResult<(), Self::Error> {
        let req = MessageRequest {
            service_code: 0x09,
            path,
            data: (),
        };
        let reply: MessageReply<()> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(())
    }

    /// invoke the Apply_Attributes service
    async fn apply_attributes<'de, D, R>(
        &mut self,
        path: EPath,
        data: D,
    ) -> StdResult<R, Self::Error>
    where
        D: Encode,
        R: Decode<'de> + 'static,
    {
        let req = MessageRequest {
            service_code: 0x0D,
            path,
            data,
        };
        let reply: MessageReply<R> = self.send(req).await?;
        Ok(reply.data)
    }

    /// invoke the Get_Attribute_Single service
    async fn get_attribute_single<'de, R>(&mut self, path: EPath) -> StdResult<R, Self::Error>
    where
        R: Decode<'de> + 'static,
    {
        let req = MessageRequest {
            service_code: 0x0E,
            path,
            data: (),
        };
        let reply: MessageReply<R> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(reply.data)
    }

    /// invoke the Set_Attribute_Single service
    async fn set_attribute_single<D: Encode>(
        &mut self,
        path: EPath,
        data: D,
    ) -> StdResult<(), Self::Error> {
        let req = MessageRequest {
            service_code: 0x10,
            path,
            data,
        };
        let reply: MessageReply<()> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(())
    }

    /// invoke the Restore service
    async fn restore(&mut self, path: EPath) -> StdResult<(), Self::Error> {
        let req = MessageRequest {
            service_code: 0x15,
            path,
            data: (),
        };
        let reply: MessageReply<()> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(())
    }

    /// invoke the Save service
    async fn save(&mut self, path: EPath) -> StdResult<(), Self::Error> {
        let req = MessageRequest {
            service_code: 0x16,
            path,
            data: (),
        };
        let reply: MessageReply<()> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(())
    }

    /// invoke the Nop service
    async fn no_operation(&mut self, path: EPath) -> StdResult<(), Self::Error> {
        let req = MessageRequest {
            service_code: 0x17,
            path,
            data: (),
        };
        let reply: MessageReply<()> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(())
    }

    /// invoke the Get_Member service
    async fn get_member<'de, R: Decode<'de> + 'static>(
        &mut self,
        path: EPath,
    ) -> StdResult<R, Self::Error> {
        let req = MessageRequest {
            service_code: 0x18,
            path,
            data: (),
        };
        let reply: MessageReply<R> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(reply.data)
    }

    /// invoke the Set_Member service
    async fn set_member<'de, D, R>(&mut self, path: EPath, data: D) -> StdResult<R, Self::Error>
    where
        D: Encode,
        R: Decode<'de> + 'static,
    {
        let req = MessageRequest {
            service_code: 0x19,
            path,
            data,
        };
        let reply: MessageReply<R> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(reply.data)
    }

    /// invoke the Insert_Member service
    async fn insert_member<'de, D, R>(&mut self, path: EPath, data: D) -> StdResult<R, Self::Error>
    where
        D: Encode,
        R: Decode<'de> + 'static,
    {
        let req = MessageRequest {
            service_code: 0x1A,
            path,
            data,
        };
        let reply: MessageReply<R> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(reply.data)
    }

    /// invoke the Remove_Member service
    async fn remove_member(&mut self, path: EPath) -> StdResult<(), Self::Error> {
        let req = MessageRequest {
            service_code: 0x1B,
            path,
            data: (),
        };
        let reply: MessageReply<()> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(())
    }

    /// invoke the Group_Sync service
    async fn group_sync(&mut self, path: EPath) -> StdResult<(), Self::Error> {
        let req = MessageRequest {
            service_code: 0x1C,
            path,
            data: (),
        };
        let reply: MessageReply<()> = self.send(req).await?;
        if !reply.status.is_ok() {
            return Err(cip_error_status(reply.status));
        }
        Ok(())
    }

    /// multiple service packet
    #[inline]
    fn multiple_service<P, D>(&mut self) -> MultipleServicePacket<'_, Self, P, D>
    where
        Self: Sized,
        P: Encode,
        D: Encode,
    {
        MultipleServicePacket::new(self)
    }
}

#[async_trait::async_trait(?Send)]
impl<T: MessageService> CommonServices for T {}
