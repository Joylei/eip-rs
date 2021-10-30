// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{codec::Encodable, frame::cip::MessageRouterRequest, Result};
use bytes::{BufMut, BytesMut};

impl<P, D> Encodable for MessageRouterRequest<P, D>
where
    P: Encodable,
    D: Encodable,
{
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        // service code
        dst.put_u8(self.service_code);

        let path_len = self.path.bytes_count();
        assert!(path_len <= u8::MAX as usize && path_len % 2 == 0);
        dst.put_u8((path_len / 2) as u8);

        self.path.encode(dst)?;
        self.data.encode(dst)?;
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        2 + self.path.bytes_count() + self.data.bytes_count()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::frame::cip::{EPath, Segment};
    use bytes::Bytes;

    #[test]
    fn test_encode_message_router_request() {
        let mr = MessageRouterRequest::new(
            0x52,
            EPath::from(vec![Segment::Class(0x06), Segment::Instance(0x01)]),
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
