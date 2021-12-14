// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::*;

pub struct SliceContainer<'a, T> {
    inner: &'a [T],
    bytes_count: Option<usize>,
}

impl<'a, T> SliceContainer<'a, T> {
    #[inline]
    pub fn new(inner: &'a [T]) -> Self {
        Self {
            inner,
            bytes_count: None,
        }
    }

    /// fast path to compute number of bytes
    #[inline]
    pub fn with_bytes_count(mut self, size: usize) -> Self {
        self.bytes_count = Some(size);
        self
    }
}

impl<'a, T> Encode for SliceContainer<'a, T>
where
    T: Encode,
{
    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        for item in self.inner.iter() {
            item.encode_by_ref(buf, encoder)?;
        }
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        if let Some(v) = self.bytes_count {
            v
        } else {
            self.inner.iter().map(|v| v.bytes_count()).sum()
        }
    }
}
