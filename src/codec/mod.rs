pub mod decode;
pub mod encode;

use crate::Result;
use crate::{
    consts::ENCAPSULATION_DATA_MAX_LEN,
    frame::{
        cip::AddressItem, encapsulation::EncapsulationPacket, CommonPacketFormat, CommonPacketItem,
        Request,
    },
};
use bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug, PartialEq)]
pub struct EIPDecoder;

#[derive(Debug, PartialEq)]
pub struct ClientCodec {
    pub(crate) decoder: EIPDecoder,
}

impl Default for ClientCodec {
    #[inline(always)]
    fn default() -> Self {
        Self {
            decoder: EIPDecoder,
        }
    }
}

pub trait Encodable {
    fn encode(self, dst: &mut BytesMut) -> Result<()>;

    /// encoded bytes count
    fn bytes_count(&self) -> usize;

    #[inline(always)]
    fn try_into_bytes(self) -> Result<Bytes>
    where
        Self: Sized,
    {
        let mut buf = BytesMut::new();
        self.encode(&mut buf)?;
        Ok(buf.freeze())
    }
}
