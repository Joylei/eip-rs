use bytes::Bytes;
use std::ops::{Deref, DerefMut};

/// item_count:u16 | item_count of CommonPacketItem
#[derive(Default, Debug)]
pub struct CommonPacketFormat(Vec<CommonPacketItem>);

impl CommonPacketFormat {
    #[inline(always)]
    pub fn into_vec(self) -> Vec<CommonPacketItem> {
        self.0
    }
}

impl Deref for CommonPacketFormat {
    type Target = [CommonPacketItem];
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CommonPacketFormat {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<CommonPacketItem>> for CommonPacketFormat {
    #[inline(always)]
    fn from(src: Vec<CommonPacketItem>) -> Self {
        Self(src)
    }
}

/// type_code:u16 | item_length:u16 | item_data
#[derive(Debug)]
pub struct CommonPacketItem {
    pub type_code: u16,
    pub data: Option<Bytes>,
}

impl CommonPacketItem {
    pub fn with_null_addr() -> Self {
        Self {
            type_code: 0,
            data: Some(Bytes::from_static(&[0x00, 0x00])),
        }
    }

    pub fn with_unconnected_data(data: Bytes) -> Self {
        Self {
            type_code: 0xB2,
            data: Some(data),
        }
    }

    pub fn with_connected_data(data: Bytes) -> Self {
        Self {
            type_code: 0xB1,
            data: Some(data),
        }
    }

    #[inline]
    pub fn is_null_addr(&self) -> bool {
        if self.type_code != 0 {
            return false;
        }
        if let Some(ref data) = self.data {
            return &data[..] == &[0x00, 0x00];
        }
        false
    }
}
