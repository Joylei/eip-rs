use crate::{
    codec::Encodable,
    frame::{cip::ConnectedSend, command::SendUnitData},
    Result,
};

impl<D: Encodable + Send> Encodable for ConnectedSend<D> {
    #[inline(always)]
    fn encode(self, dst: &mut bytes::BytesMut) -> Result<()> {
        let command = SendUnitData {
            connection_id: self.connection_id,
            session_handle: self.session_handle,
            sequence_number: self.sequence_number,
            data: self.data,
        };
        command.encode(dst)?;

        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        let addr_len = if self.sequence_number.is_some() {
            12
        } else {
            8
        };
        24 // encapsulation header
        + 4 // handle,
        + 2 // timeout
        + 2// item count
        + addr_len // address item
        + 2 // item type
        + 2 // item data length
        + self.data.bytes_count()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_encode_connected_send() {
        let mut request = ConnectedSend::new(Bytes::from_static(&[0x10, 0x00, 0x88]));
        request.session_handle = 0x11;
        request.connection_id = 0x23;
        assert_eq!(request.bytes_count(), 47);
        let buf = request.try_into_bytes().unwrap();
        assert_eq!(
            &buf[..],
            &[
                0x70, 0x00, // command
                0x17, 0x00, // data len, 23
                0x11, 0x00, 0x00, 0x00, // session handle
                0x00, 0x00, 0x00, 0x00, // status
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // sender context
                0x00, 0x00, 0x00, 0x00, // options
                // - encapsulation data -
                0x00, 0x00, 0x00, 0x00, // interface handle
                0x00, 0x00, // timeout
                // -- cpf --
                0x02, 0x00, // cpf item count
                0xA1, 0x00, // connected address
                0x04, 0x00, // addr len
                0x23, 0x00, 0x00, 0x00, // connection id
                0xB1, 0x00, // data item type
                0x03, 0x00, // data item len, 20
                0x10, 0x00, 0x88, // mr data
            ]
        );
    }

    #[test]
    fn test_encode_sequenced_send() {
        let mut request = ConnectedSend::new(Bytes::from_static(&[0x10, 0x00, 0x88]));
        request.session_handle = 0x11;
        request.connection_id = 0x23;
        request.sequence_number = Some(0x21);
        assert_eq!(request.bytes_count(), 51);
        let buf = request.try_into_bytes().unwrap();
        assert_eq!(
            &buf[..],
            &[
                0x70, 0x00, // command
                0x1B, 0x00, // data len, 27
                0x11, 0x00, 0x00, 0x00, // session handle
                0x00, 0x00, 0x00, 0x00, // status
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // sender context
                0x00, 0x00, 0x00, 0x00, // options
                // - encapsulation data -
                0x00, 0x00, 0x00, 0x00, // interface handle
                0x00, 0x00, // timeout
                // -- cpf --
                0x02, 0x00, // cpf item count
                0x02, 0x80, // sequenced address
                0x08, 0x00, // addr len
                0x23, 0x00, 0x00, 0x00, // connection id
                0x21, 0x00, 0x00, 0x00, // sequence number
                0xB1, 0x00, // data item type
                0x03, 0x00, // data item len, 20
                0x10, 0x00, 0x88, // mr data
            ]
        );
    }
}
