// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod epath;
mod message;

use crate::{
    connection::{ConnectionParameters, ForwardCloseRequest, OpenOptions},
    service::request::UnconnectedSend,
    MessageRequest,
};
use bytes::{BufMut, BytesMut};
use rseip_core::codec::{Encode, Encoder};

impl<P: Encode> OpenOptions<P> {
    #[inline]
    fn encode_common<A: Encoder>(
        &self,
        dst: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
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
        Self::encode_parameters(self.large_open, &self.o_t_params, dst, encoder)?;
        dst.put_u32_le(self.t_o_rpi);
        Self::encode_parameters(self.large_open, &self.t_o_params, dst, encoder)?;

        dst.put_u8(transport_class_trigger);

        let path_len = self.connection_path.bytes_count();
        assert!(path_len % 2 == 0 && path_len <= u8::MAX as usize);
        dst.put_u8((path_len / 2) as u8);

        Ok(())
    }

    #[inline]
    fn encode_parameters<A: Encoder>(
        large: bool,
        parameters: &ConnectionParameters,
        dst: &mut BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error> {
        if large {
            let mut v = parameters.connection_size as u32;
            v |= (parameters.variable_length as u32) << 25;
            v |= (parameters.priority as u32) << 26;
            v |= (parameters.connection_type as u32) << 29;
            v |= (parameters.redundant_owner as u32) << 31;
            dst.put_u32_le(v);
        } else {
            let mut v = parameters.connection_size & 0x01FF;
            v |= (parameters.variable_length as u16) << 9;
            v |= (parameters.priority as u16) << 10;
            v |= (parameters.connection_type as u16) << 13;
            v |= (parameters.redundant_owner as u16) << 15;
            dst.put_u16_le(v);
        }
        Ok(())
    }
}

impl<P: Encode> Encode for OpenOptions<P> {
    #[inline]
    fn encode<A: Encoder>(self, dst: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error> {
        self.encode_common(dst, encoder)?;
        self.connection_path.encode(dst, encoder)?;
        Ok(())
    }

    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        dst: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        self.encode_common(dst, encoder)?;
        self.connection_path.encode_by_ref(dst, encoder)?;
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        let base_size = if self.large_open { 40 } else { 36 };
        base_size + self.connection_path.bytes_count()
    }
}

impl<P: Encode> ForwardCloseRequest<P> {
    #[inline]
    fn encode_common<A: Encoder>(
        &self,
        dst: &mut BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error> {
        dst.put_u8(self.priority_time_ticks);
        dst.put_u8(self.timeout_ticks);
        dst.put_u16_le(self.connection_serial_number);
        dst.put_u16_le(self.originator_vendor_id);
        dst.put_u32_le(self.originator_serial_number);

        let path_len = self.connection_path.bytes_count();
        assert!(path_len % 2 == 0 && path_len <= u8::MAX as usize);
        dst.put_u8(path_len as u8 / 2); //path size
        dst.put_u8(0); // reserved

        Ok(())
    }
}

impl<P: Encode> Encode for ForwardCloseRequest<P> {
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error> {
        self.encode_common(buf, encoder)?;
        self.connection_path.encode(buf, encoder)?;
        Ok(())
    }

    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        self.encode_common(buf, encoder)?;
        self.connection_path.encode_by_ref(buf, encoder)?;
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        12 + self.connection_path.bytes_count()
    }
}

impl<CP, P, D> Encode for UnconnectedSend<CP, MessageRequest<P, D>>
where
    CP: Encode,
    P: Encode,
    D: Encode,
{
    #[inline]
    fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> Result<(), A::Error>
    where
        Self: Sized,
    {
        let data_len = self.data.bytes_count();

        buf.put_u8(self.priority_ticks);
        buf.put_u8(self.timeout_ticks);

        buf.put_u16_le(data_len as u16); // size of MR
        self.data.encode(buf, encoder)?;
        if data_len % 2 == 1 {
            buf.put_u8(0); // padded 0
        }

        let path_len = self.path.bytes_count();
        buf.put_u8(path_len as u8 / 2); // path size in words
        buf.put_u8(0); // reserved
        self.path.encode(buf, encoder)?; // padded epath
        Ok(())
    }

    #[inline]
    fn encode_by_ref<A: Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        let data_len = self.data.bytes_count();

        buf.put_u8(self.priority_ticks);
        buf.put_u8(self.timeout_ticks);

        buf.put_u16_le(data_len as u16); // size of MR
        self.data.encode_by_ref(buf, encoder)?;
        if data_len % 2 == 1 {
            buf.put_u8(0); // padded 0
        }

        let path_len = self.path.bytes_count();
        buf.put_u8(path_len as u8 / 2); // path size in words
        buf.put_u8(0); // reserved
        self.path.encode_by_ref(buf, encoder)?; // padded epath
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        let data_len = self.data.bytes_count();
        4 + data_len + data_len % 2 + 2 + self.path.bytes_count()
    }
}
