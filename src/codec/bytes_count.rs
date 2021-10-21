use crate::frame::{CommonPacketFormat, CommonPacketItem};

use super::EncodedBytesCount;

impl EncodedBytesCount for CommonPacketFormat {
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        let count: usize = self.iter().map(|v| v.bytes_count()).sum();
        count + 2
    }
}

impl EncodedBytesCount for CommonPacketItem {
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        4 + self.data.as_ref().map(|v| v.len()).unwrap_or_default()
    }
}
