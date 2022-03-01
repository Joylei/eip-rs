// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::MessageRequest;
use bytes::BufMut;
use rseip_core::codec::{Encode, Encoder};

impl<P: Encode, D: Encode> Encode for MessageRequest<P, D> {
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut bytes::BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        // service code
        buf.put_u8(self.service_code);

        let path_len = self.path.bytes_count();
        debug_assert!(path_len <= u8::MAX as usize && path_len % 2 == 0);
        buf.put_u8((path_len / 2) as u8);

        self.path.encode(buf, encoder)?;
        self.data.encode(buf, encoder)?;
        Ok(())
    }

    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut bytes::BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        // service code
        buf.put_u8(self.service_code);

        let path_len = self.path.bytes_count();
        debug_assert!(path_len <= u8::MAX as usize && path_len % 2 == 0);
        buf.put_u8((path_len / 2) as u8);

        self.path.encode_by_ref(buf, encoder)?;
        self.data.encode_by_ref(buf, encoder)?;
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        2 + self.path.bytes_count() + self.data.bytes_count()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{epath::EPath, MessageRequest};
    use bytes::Bytes;
    use rseip_core::tests::EncodeExt;

    #[test]
    fn test_encode_message_router_request() {
        let mr = MessageRequest::new(
            0x52,
            EPath::default().with_class(0x06).with_instance(0x01),
            Bytes::from_static(&[0x10, 0x00]),
        );
        assert_eq!(mr.bytes_count(), 8);
        let buf = mr.try_into_bytes().unwrap();
        assert_eq!(
            &buf[..],
            &[
                0x52, // service code
                0x02, 0x20, 0x06, 0x24, 0x01, // epath
                0x010, 0x00 // data
            ]
        );
    }
}
