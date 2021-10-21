use std::ops::{Deref, DerefMut};

use bytes::Bytes;

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
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<CommonPacketItem>> for CommonPacketFormat {
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
