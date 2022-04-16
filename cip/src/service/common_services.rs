// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod multiple_packet;

use super::*;
use crate::epath::EPath;
pub use multiple_packet::MultipleServicePacket;
use rseip_core::codec::{Decode, Encode, SliceContainer};

/// common services
#[async_trait::async_trait]
pub trait CommonServices: MessageService {
    /// invoke the Get_Attribute_All service
    #[inline]
    async fn get_attribute_all<'de, R>(&mut self, path: EPath) -> Result<R, Self::Error>
    where
        R: Decode<'de> + 'static,
    {
        send_and_extract(self, 0x01, path, ()).await
    }

    /// invoke the Set_Attribute_All service
    #[inline]
    async fn set_attribute_all<D: Encode + Send + Sync>(
        &mut self,
        path: EPath,
        attrs: D,
    ) -> Result<(), Self::Error> {
        send_and_extract(self, 0x02, path, attrs).await
    }

    /// invoke the Get_Attribute_List
    #[inline]
    async fn get_attribute_list<'de, R>(
        &mut self,
        path: EPath,
        attrs: &[u16],
    ) -> Result<R, Self::Error>
    where
        R: Decode<'de> + 'static,
    {
        let attrs_len = attrs.len();
        debug_assert!(attrs_len <= u16::MAX as usize);
        send_and_extract(
            self,
            0x03,
            path,
            (
                attrs_len as u16,
                SliceContainer::new(attrs).with_bytes_count(2 * attrs_len),
            ),
        )
        .await
    }

    /// invoke the Set_Attribute_List service
    #[inline]
    async fn set_attribute_list<'de, D, R>(
        &mut self,
        path: EPath,
        attrs: D,
    ) -> Result<R, Self::Error>
    where
        D: Encode + Send + Sync,
        R: Decode<'de> + 'static,
    {
        send_and_extract(self, 0x04, path, attrs).await
    }

    /// invoke the Reset service
    #[inline]
    async fn reset(&mut self, path: EPath) -> Result<(), Self::Error> {
        send_and_extract(self, 0x05, path, ()).await
    }

    /// invoke the Start service
    #[inline]
    async fn start(&mut self, path: EPath) -> Result<(), Self::Error> {
        send_and_extract(self, 0x06, path, ()).await
    }

    /// invoke the Stop service
    #[inline]
    async fn stop(&mut self, path: EPath) -> Result<(), Self::Error> {
        send_and_extract(self, 0x07, path, ()).await
    }

    /// invoke the Create service
    #[inline]
    async fn create<'de, D, R>(&mut self, path: EPath, data: D) -> Result<R, Self::Error>
    where
        D: Encode + Send + Sync,
        R: Decode<'de> + 'static,
    {
        send_and_extract(self, 0x08, path, data).await
    }

    /// invoke the Delete service
    #[inline]
    async fn delete(&mut self, path: EPath) -> Result<(), Self::Error> {
        send_and_extract(self, 0x09, path, ()).await
    }

    /// invoke the Apply_Attributes service
    #[inline]
    async fn apply_attributes<'de, D, R>(&mut self, path: EPath, data: D) -> Result<R, Self::Error>
    where
        D: Encode + Send + Sync,
        R: Decode<'de> + 'static,
    {
        send_and_extract(self, 0x0D, path, data).await
    }

    /// invoke the Get_Attribute_Single service
    #[inline]
    async fn get_attribute_single<'de, R>(&mut self, path: EPath) -> Result<R, Self::Error>
    where
        R: Decode<'de> + 'static,
    {
        send_and_extract(self, 0x0E, path, ()).await
    }

    /// invoke the Set_Attribute_Single service
    #[inline]
    async fn set_attribute_single<D: Encode + Send + Sync>(
        &mut self,
        path: EPath,
        data: D,
    ) -> Result<(), Self::Error> {
        send_and_extract(self, 0x10, path, data).await
    }

    /// invoke the Restore service
    #[inline]
    async fn restore(&mut self, path: EPath) -> Result<(), Self::Error> {
        send_and_extract(self, 0x15, path, ()).await
    }

    /// invoke the Save service
    #[inline]
    async fn save(&mut self, path: EPath) -> Result<(), Self::Error> {
        send_and_extract(self, 0x16, path, ()).await
    }

    /// invoke the Nop service
    #[inline]
    async fn no_operation(&mut self, path: EPath) -> Result<(), Self::Error> {
        send_and_extract(self, 0x17, path, ()).await
    }

    /// invoke the Get_Member service
    #[inline]
    async fn get_member<'de, R: Decode<'de> + 'static>(
        &mut self,
        path: EPath,
    ) -> Result<R, Self::Error> {
        send_and_extract(self, 0x18, path, ()).await
    }

    /// invoke the Set_Member service
    #[inline]
    async fn set_member<'de, D, R>(&mut self, path: EPath, data: D) -> Result<R, Self::Error>
    where
        D: Encode + Send + Sync,
        R: Decode<'de> + 'static,
    {
        send_and_extract(self, 0x19, path, data).await
    }

    /// invoke the Insert_Member service
    #[inline]
    async fn insert_member<'de, D, R>(&mut self, path: EPath, data: D) -> Result<R, Self::Error>
    where
        D: Encode + Send + Sync,
        R: Decode<'de> + 'static,
    {
        send_and_extract(self, 0x1A, path, data).await
    }

    /// invoke the Remove_Member service
    #[inline]
    async fn remove_member(&mut self, path: EPath) -> Result<(), Self::Error> {
        send_and_extract(self, 0x1B, path, ()).await
    }

    /// invoke the Group_Sync service
    #[inline]
    async fn group_sync(&mut self, path: EPath) -> Result<(), Self::Error> {
        send_and_extract(self, 0x1C, path, ()).await
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

#[async_trait::async_trait]
impl<T: MessageService> CommonServices for T {}
