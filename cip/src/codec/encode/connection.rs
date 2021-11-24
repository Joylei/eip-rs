// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::{
    codec::Encodable,
    connection::{ConnectionParameters, ForwardCloseRequest, Options},
    Result,
};
use bytes::{BufMut, BytesMut};

impl<P: Encodable> Encodable for Options<P> {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        let encode_parameters =
            move |large: bool, parameters: ConnectionParameters, dst: &mut BytesMut| {
                if large {
                    let mut v = parameters.connection_size as u32;
                    v = v | ((parameters.variable_length as u32) << 25);
                    v = v | ((parameters.priority as u32) << 26);
                    v = v | ((parameters.connection_type as u32) << 29);
                    v = v | ((parameters.redundant_owner as u32) << 31);
                    dst.put_u32_le(v);
                } else {
                    let mut v = parameters.connection_size & 0x01FF;
                    v = v | ((parameters.variable_length as u16) << 9);
                    v = v | ((parameters.priority as u16) << 10);
                    v = v | ((parameters.connection_type as u16) << 13);
                    v = v | ((parameters.redundant_owner as u16) << 15);
                    dst.put_u16_le(v);
                }
            };

        let transport_class_trigger = self.transport_class_trigger();

        dst.put_u8(self.priority_tick_time);
        dst.put_u8(self.timeout_ticks);
        dst.put_u32_le(self.o_t_connection_id);
        dst.put_u32_le(self.t_o_connection_id);
        dst.put_u16_le(self.connection_serial_number);
        dst.put_u16_le(self.vendor_id);
        dst.put_u32_le(self.originator_serial_number);
        dst.put_u8(self.timeout_multiplier);
        dst.put_slice(&[0, 0, 0]); //  reserved
        dst.put_u32_le(self.o_t_rpi);
        encode_parameters(self.large_open, self.o_t_params, dst);
        dst.put_u32_le(self.t_o_rpi);
        encode_parameters(self.large_open, self.t_o_params, dst);

        dst.put_u8(transport_class_trigger);

        let path_len = self.connection_path.bytes_count();
        assert!(path_len % 2 == 0 && path_len <= u8::MAX as usize);
        dst.put_u8((path_len / 2) as u8);
        self.connection_path.encode(dst)?;
        Ok(())
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        let base_size = if self.large_open { 40 } else { 36 };
        base_size + self.connection_path.bytes_count()
    }
}

impl<P: Encodable> Encodable for ForwardCloseRequest<P> {
    #[inline(always)]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        dst.put_u8(self.priority_time_ticks);
        dst.put_u8(self.timeout_ticks);
        dst.put_u16_le(self.connection_serial_number);
        dst.put_u16_le(self.originator_vendor_id);
        dst.put_u32_le(self.originator_serial_number);

        let path_len = self.connection_path.bytes_count();

        assert!(path_len % 2 == 0 && path_len <= u8::MAX as usize);
        dst.put_u8(path_len as u8 / 2); //path size
        dst.put_u8(0); // reserved
        self.connection_path.encode(dst)?;
        Ok(())
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        12 + self.connection_path.bytes_count()
    }
}
