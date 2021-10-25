use crate::{
    codec::Encodable,
    frame::{cip::ConnectedSend, command::SendUnitData},
    Result,
};

impl<D: Encodable + Send> Encodable for ConnectedSend<D> {
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
