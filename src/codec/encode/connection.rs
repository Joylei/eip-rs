use crate::{
    codec::Encodable,
    frame::cip::connection::{ConnectionParameters, ForwardCloseRequest},
    Result,
};
use bytes::{BufMut, BytesMut};

impl<P: Encodable> Encodable for ConnectionParameters<P> {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        dst.put_u8(self.priority_time_ticks);
        dst.put_u8(self.timeout_ticks);
        dst.put_u32_le(self.o_t_connection_id);
        dst.put_u32_le(self.t_o_connection_id);
        dst.put_u16_le(self.connection_serial_number);
        dst.put_u16_le(self.vendor_id);
        dst.put_u32_le(self.originator_serial_number);
        dst.put_u8(self.timeout_multiplier);
        dst.put_slice(&[0, 0, 0]); //  reserved
        dst.put_u32_le(self.o_t_rpi);
        dst.put_u16_le(self.o_t_connection_parameters);
        dst.put_u32_le(self.t_o_rpi);
        dst.put_u16_le(self.t_o_connection_parameters);
        dst.put_u8(self.transport_class_trigger);

        let path_len = self.connection_path.bytes_count();
        assert!(path_len % 2 == 0 && path_len <= u8::MAX as usize);
        dst.put_u8((path_len / 2) as u8);
        self.connection_path.encode(dst)?;
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        36 + self.connection_path.bytes_count()
    }
}

impl<P: Encodable> Encodable for ForwardCloseRequest<P> {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        dst.put_u8(self.priority_time_ticks);
        dst.put_u8(self.timeout_tick);
        dst.put_u16_le(self.connection_serial_number);
        dst.put_u16_le(self.originator_vender_id);
        dst.put_u16_le(self.connection_serial_number);

        let path_len = self.connection_path.bytes_count();

        assert!(path_len % 2 == 0 && path_len <= u8::MAX as usize);
        dst.put_u8(path_len as u8 / 2); //path size
        dst.put_u8(0); // reserved
        self.connection_path.encode(dst)?;
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        10 + self.connection_path.bytes_count()
    }
}
