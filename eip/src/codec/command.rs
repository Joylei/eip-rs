// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::*;
use crate::{command::*, consts::*, EncapsulationPacket, Error};
use bytes::{BufMut, BytesMut};
use tokio_util::codec::Encoder;

impl<C: Command, E: Error> Encoder<C> for ClientCodec<E> {
    type Error = E;
    #[inline(always)]
    fn encode(&mut self, item: C, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.encode(dst, self)
    }
}

impl<D: Encode> Encode for command::Nop<D> {
    #[inline]
    fn encode<A: codec::Encoder>(
        self,
        dst: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        // reserve space for performance
        dst.reserve(self.bytes_count());

        let mut pkt = EncapsulationPacket {
            hdr: Default::default(),
            data: self.data,
        };
        pkt.hdr.command = Self::command_code();
        pkt.encode(dst, encoder)
    }

    #[inline]
    fn encode_by_ref<A: codec::Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        // reserve space for performance
        buf.reserve(self.bytes_count());

        let mut pkt = EncapsulationPacket {
            hdr: Default::default(),
            data: &self.data,
        };
        pkt.hdr.command = Self::command_code();
        pkt.encode(buf, encoder)
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN + self.data.bytes_count()
    }
}

impl Encode for ListIdentity {
    #[inline]
    fn encode_by_ref<A: codec::Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        let mut pkt = EncapsulationPacket::default();
        pkt.hdr.command = Self::command_code();
        pkt.data = ();
        pkt.encode(buf, encoder)
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN
    }
}

impl Encode for ListInterfaces {
    #[inline]
    fn encode_by_ref<A: codec::Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        let mut pkt = EncapsulationPacket::default();
        pkt.hdr.command = Self::command_code();
        pkt.data = ();
        pkt.encode(buf, encoder)
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN
    }
}

impl Encode for ListServices {
    #[inline]
    fn encode_by_ref<A: codec::Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        let mut pkt = EncapsulationPacket::default();
        pkt.hdr.command = Self::command_code();
        pkt.data = ();
        pkt.encode(buf, encoder)
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN
    }
}

impl Encode for RegisterSession {
    #[inline]
    fn encode_by_ref<A: codec::Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        let mut pkt: EncapsulationPacket<&[u8]> = EncapsulationPacket::default();
        pkt.hdr.command = Self::command_code();
        pkt.data = &[0x01, 0x00, 0x00, 0x00];
        pkt.encode(buf, encoder)
    }

    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN + 4
    }
}

impl Encode for UnRegisterSession {
    #[inline]
    fn encode_by_ref<A: codec::Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        let mut pkt = EncapsulationPacket::default();
        pkt.hdr.command = Self::command_code();
        pkt.hdr.session_handle = self.session_handle;
        pkt.data = ();
        pkt.encode(buf, encoder)
    }
    #[inline(always)]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN
    }
}

impl<D: Encode> Encode for SendRRData<D> {
    #[inline]
    fn encode<A: codec::Encoder>(
        self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        let mut pkt = EncapsulationPacket {
            hdr: Default::default(),
            data: UnconnectedData {
                timeout: self.timeout,
                data: self.data,
            },
        };
        pkt.hdr.command = Self::command_code();
        pkt.hdr.session_handle = self.session_handle;
        pkt.encode(buf, encoder)
    }

    #[inline]
    fn encode_by_ref<A: codec::Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        let mut pkt = EncapsulationPacket {
            hdr: Default::default(),
            data: UnconnectedData {
                timeout: self.timeout,
                data: &self.data,
            },
        };
        pkt.hdr.command = Self::command_code();
        pkt.hdr.session_handle = self.session_handle;
        pkt.encode(buf, encoder)
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN + 16 + self.data.bytes_count()
    }
}

impl<D: Encode> Encode for SendUnitData<D> {
    #[inline]
    fn encode<A: codec::Encoder>(
        self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        let packed_data = ConnectedData {
            connection_id: self.connection_id,
            sequence_number: self.sequence_number,
            data: self.data,
        };
        let mut pkt = EncapsulationPacket {
            hdr: Default::default(),
            data: packed_data,
        };
        pkt.hdr.command = Self::command_code();
        pkt.hdr.session_handle = self.session_handle;
        pkt.encode(buf, encoder)
    }

    #[inline]
    fn encode_by_ref<A: codec::Encoder>(
        &self,
        buf: &mut BytesMut,
        encoder: &mut A,
    ) -> Result<(), A::Error> {
        let packed_data = ConnectedData {
            connection_id: self.connection_id,
            sequence_number: self.sequence_number,
            data: &self.data,
        };
        let mut pkt = EncapsulationPacket {
            hdr: Default::default(),
            data: packed_data,
        };
        pkt.hdr.command = Self::command_code();
        pkt.hdr.session_handle = self.session_handle;
        pkt.encode(buf, encoder)
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        ENCAPSULATION_HEADER_LEN + 22 + self.data.bytes_count()
    }
}

struct UnconnectedData<D> {
    timeout: u16,
    data: D,
}

impl<D: Encode> UnconnectedData<D> {
    #[inline]
    fn put_common<A: codec::Encoder>(
        &self,
        buf: &mut BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error> {
        buf.put_u32_le(0); // interface handle, shall be 0 for CIP
        buf.put_u16_le(self.timeout); // timeout, 0 for SendUnitData
        buf.put_u16_le(2); //  cpf item count
        buf.put_slice(&[0, 0, 0, 0]); // null address
        buf.put_u16_le(0xB2); // unconnected data item
        buf.put_u16_le(self.data.bytes_count() as u16);
        Ok(())
    }
}

struct ConnectedData<D> {
    connection_id: u32,
    sequence_number: u16,
    data: D,
}

impl<D: Encode> ConnectedData<D> {
    #[inline]
    fn put_common<A: codec::Encoder>(
        &self,
        buf: &mut BytesMut,
        _encoder: &mut A,
    ) -> Result<(), A::Error> {
        buf.put_u32_le(0); // interface handle, shall be 0 for CIP
        buf.put_u16_le(0); // timeout, 0 for SendUnitData
        buf.put_u16_le(2); //  cpf item count

        buf.put_u16_le(0xA1); // connected address item
        buf.put_u16_le(4); // data len
        buf.put_u32_le(self.connection_id);

        buf.put_u16_le(0xB1); // connected data item
        buf.put_u16_le(self.data.bytes_count() as u16 + 2); // data item len
        buf.put_u16_le(self.sequence_number);
        Ok(())
    }
}

macro_rules! impl_data_encode {
    ($ty:ident,$cnt:tt) => {
        impl<D: Encode> Encode for $ty<D> {
            #[inline]
            fn encode<A: codec::Encoder>(
                self,
                buf: &mut BytesMut,
                encoder: &mut A,
            ) -> Result<(), A::Error>
            where
                Self: Sized,
            {
                self.put_common(buf, encoder)?;
                self.data.encode_by_ref(buf, encoder)?; // data request
                Ok(())
            }

            #[inline]
            fn encode_by_ref<A: codec::Encoder>(
                &self,
                buf: &mut BytesMut,
                encoder: &mut A,
            ) -> Result<(), A::Error> {
                self.put_common(buf, encoder)?;
                self.data.encode_by_ref(buf, encoder)?; // data request
                Ok(())
            }

            fn bytes_count(&self) -> usize {
                $cnt + self.data.bytes_count()
            }
        }
    };
}

impl_data_encode!(UnconnectedData, 16);
impl_data_encode!(ConnectedData, 22);

#[cfg(test)]
mod test {
    use super::*;
    use rseip_core::tests::EncodeExt;

    #[test]
    fn test_list_identity_request() {
        let buf = ListIdentity.try_into_bytes().unwrap();
        assert_eq!(
            &buf[..],
            &[
                0x63, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            ]
        )
    }
}
