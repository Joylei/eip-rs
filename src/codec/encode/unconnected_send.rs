use crate::{
    codec::{encode::LazyEncode, Encodable},
    frame::{
        cip::{MessageRouterRequest, UnconnectedSend},
        command::SendRRData,
    },
    Result,
};
use bytes::BufMut;

impl<D: Encodable + Send> Encodable for UnconnectedSend<D> {
    #[inline]
    fn encode(self, dst: &mut bytes::BytesMut) -> Result<()> {
        let Self {
            session_handle,
            priority_ticks,
            timeout_ticks,
            timeout,
            path: route_path,
            data: mr_data,
        } = self;
        let mr_data_len = mr_data.bytes_count();
        let path_len = route_path.bytes_count();

        assert!(mr_data_len <= u16::MAX as usize);
        debug_assert!(path_len % 2 == 0);
        assert!(path_len <= u8::MAX as usize);

        let unconnected_send: MessageRouterRequest<&[u8], _> = MessageRouterRequest {
            service_code: 0x52,
            path: &[0x20, 0x06, 0x24, 0x01],
            data: LazyEncode {
                f: move |buf: &mut bytes::BytesMut| {
                    buf.put_u8(priority_ticks);
                    buf.put_u8(timeout_ticks);

                    buf.put_u16_le(mr_data_len as u16); // size of MR
                    mr_data.encode(buf)?;
                    if mr_data_len % 2 == 1 {
                        buf.put_u8(0); // padded 0
                    }

                    buf.put_u8(path_len as u8 / 2); // path size in words
                    buf.put_u8(0); // reserved
                    route_path.encode(buf)?; // padded epath
                    Ok(())
                },
                bytes_count: 8 + mr_data_len + mr_data_len % 2 + path_len,
            },
        };
        let command = SendRRData {
            session_handle,
            timeout,
            data: unconnected_send,
        };
        command.encode(dst)?;
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        let mr_data_len = self.data.bytes_count();
        24 // encapsulation header
        + 4 // handle,
        + 2 // timeout
        + 2// item count
        + 4 // null address
        + 2 // item type
        + 2 // item data length
        + 2 + 4 // unconnected send service code + path
        + 2 // time ticks
        +  2 // mr service size
        + mr_data_len + mr_data_len % 2 // padded if need
        + 2 // path len + reserved 
        + self.path.bytes_count()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::frame::cip::{EPath, PortSegment, Segment};
    use bytes::Bytes;

    #[test]
    fn test_encode_unconnected_send() {
        let ucmm = UnconnectedSend::new(
            EPath::from(vec![Segment::Port(PortSegment::default())]),
            Bytes::from_static(&[0x10, 0x00, 0x88]),
        );
        assert_eq!(ucmm.bytes_count(), 58);
        let buf = ucmm.try_into_bytes().unwrap();
        assert_eq!(
            &buf[..],
            &[
                0x6F, 0x00, // command
                0x24, 0x00, // data len, 36
                0x00, 0x00, 0x00, 0x00, // session handle
                0x00, 0x00, 0x00, 0x00, // status
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // sender context
                0x00, 0x00, 0x00, 0x00, // options
                // - encapsulation data -
                0x00, 0x00, 0x00, 0x00, // interface handle
                0x00, 0x00, // timeout
                // -- cpf --
                0x02, 0x00, // cpf item count
                0x00, 0x00, 0x00, 0x00, // null address
                0xB2, 0x00, // data item type
                0x14, 0x00, // data item len, 20
                0x52, // cm service
                0x02, 0x20, 0x06, 0x24, 0x01, // path
                0x03, 0xFA, // time ticks
                0x03, 0x00, // mr size in bytes
                0x10, 0x00, 0x88, // mr data
                0x00, //padded
                0x01, 0x00, 0x01, 0x00 // connection path
            ]
        );
    }

    #[test]
    fn test_encode_unconnected_send_padded() {
        let ucmm = UnconnectedSend::new(
            EPath::from(vec![Segment::Port(PortSegment::default())]),
            Bytes::from_static(&[0x10, 0x00, 0x88, 0x99]),
        );
        assert_eq!(ucmm.bytes_count(), 58);
        let buf = ucmm.try_into_bytes().unwrap();
        assert_eq!(
            &buf[..],
            &[
                0x6F, 0x00, // command
                0x24, 0x00, // data len, 36
                0x00, 0x00, 0x00, 0x00, // session handle
                0x00, 0x00, 0x00, 0x00, // status
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // sender context
                0x00, 0x00, 0x00, 0x00, // options
                // - encapsulation data -
                0x00, 0x00, 0x00, 0x00, // interface handle
                0x00, 0x00, // timeout
                // -- cpf --
                0x02, 0x00, // cpf item count
                0x00, 0x00, 0x00, 0x00, // null address
                0xB2, 0x00, // data item type
                0x14, 0x00, // data item len, 20
                0x52, // cm service
                0x02, 0x20, 0x06, 0x24, 0x01, // path
                0x03, 0xFA, // time ticks
                0x04, 0x00, // mr size in bytes
                0x10, 0x00, 0x88, 0x99, // mr data
                0x01, 0x00, 0x01, 0x00 // connection path
            ]
        );
    }
}
