use super::instance::GetInstanceAttributeList;
use super::*;
use crate::cip::codec::LazyEncode;
use crate::Error;
use crate::{cip, error::InnerError};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, BytesMut};
use cip::CipError;
use std::convert::TryFrom;

/// AB related operations
#[async_trait::async_trait(?Send)]
pub trait AbService {
    /// Read Tag Service,
    /// CIP Data Table Read
    async fn read_tag<P, R>(&mut self, req: P) -> Result<R>
    where
        P: Into<TagRequest>,
        R: TryFrom<Bytes>,
        R::Error: Into<crate::Error>;

    /// Write Tag Service,
    /// CIP Data Table Write
    async fn write_tag<P, D>(&mut self, req: P, value: TagValue<D>) -> Result<()>
    where
        P: Into<TagRequest>,
        D: Encodable;

    /// Read Tag Fragmented Service, enables client applications to read a tag
    /// with data that does not fit into a single packet (approximately 500 bytes)
    async fn read_tag_fragmented<F, R>(
        &mut self,
        req: ReadFragmentedRequest<F, R>,
    ) -> Result<(bool, R)>
    where
        F: Fn(u16, Bytes) -> Result<R>;

    /// Write Tag Fragmented Service, enables client applications to write to a tag
    /// in the controller whose data will not fit into a single packet (approximately 500 bytes)
    async fn write_tag_fragmented<D: Encodable>(
        &mut self,
        req: WriteFragmentedRequest<D>,
    ) -> Result<bool>;

    /// Read Modify Write Tag Service, modifies Tag data with individual bit resolution
    async fn read_modify_write<const N: usize>(
        &mut self,
        req: ReadModifyWriteRequest<N>,
    ) -> Result<()>;

    fn get_instance_attribute_list(&mut self) -> GetInstanceAttributeList<Self>
    where
        Self: Sized;
}

macro_rules! impl_service {
    ($t:ty) => {
        #[async_trait::async_trait(?Send)]
        impl AbService for $t {
            /// Read Tag Service,
            /// CIP Data Table Read
            #[inline]
            async fn read_tag<P, R>(&mut self, req: P) -> Result<R>
            where
                P: Into<TagRequest>,
                R: TryFrom<Bytes>,
                R::Error: Into<crate::Error>,
            {
                let res = ab_read_tag(self, req).await?;
                Ok(res)
            }

            /// Write Tag Service,
            /// CIP Data Table Write
            #[inline]
            async fn write_tag<P, D>(&mut self, req: P, value: TagValue<D>) -> Result<()>
            where
                P: Into<TagRequest>,
                D: Encodable,
            {
                ab_write_tag(self, req, value).await?;
                Ok(())
            }

            /// Read Tag Fragmented Service
            #[inline]
            async fn read_tag_fragmented<F, R>(
                &mut self,
                req: ReadFragmentedRequest<F, R>,
            ) -> Result<(bool, R)>
            where
                F: Fn(u16, Bytes) -> Result<R>,
            {
                let res = ab_read_tag_fragmented(self, req).await?;
                Ok(res)
            }

            /// Write Tag Fragmented Service, enables client applications to write to a tag
            /// in the controller whose data will not fit into a single packet (approximately 500 bytes)
            #[inline]
            async fn write_tag_fragmented<D: Encodable>(
                &mut self,
                req: WriteFragmentedRequest<D>,
            ) -> Result<bool> {
                let res = ab_write_tag_fragmented(self, req).await?;
                Ok(res)
            }

            /// Read Modify Write Tag Service, modifies Tag data with individual bit resolution
            #[inline]
            async fn read_modify_write<const N: usize>(
                &mut self,
                req: ReadModifyWriteRequest<N>,
            ) -> Result<()> {
                ab_read_modify_write(self, req).await?;
                Ok(())
            }

            #[inline]
            fn get_instance_attribute_list(&mut self) -> GetInstanceAttributeList<Self>
            where
                Self: Sized,
            {
                GetInstanceAttributeList::new(self)
            }
        }
    };
}

impl_service!(AbEipClient);
impl_service!(AbEipConnection);
impl_service!(MaybeConnected<AbEipDriver>);

/// Read Tag Service,
/// CIP Data Table Read
async fn ab_read_tag<C: MessageService<Error = Error>, P, R>(client: &mut C, req: P) -> Result<R>
where
    P: Into<TagRequest>,
    R: TryFrom<Bytes>,
    R::Error: Into<crate::Error>,
{
    let req: TagRequest = req.into();
    let mr_request = MessageRequest::new(0x4C, req.tag, ElementCount(req.count));
    let resp = client.send(mr_request).await?;
    if resp.reply_service != 0xCC {
        return Err(rseip_core::Error::<InnerError>::from_invalid_data()
            .with_context("unexpected reply for read tag service")
            .into());
    }
    if resp.status.general != 0 {
        return Err(CipError::Cip(resp.status).into());
    }
    R::try_from(resp.data).map_err(|e| e.into())
}

/// Write Tag Service,
/// CIP Data Table Write
async fn ab_write_tag<C: MessageService<Error = Error>, P, D>(
    client: &mut C,
    req: P,
    value: TagValue<D>,
) -> Result<()>
where
    P: Into<TagRequest>,
    D: Encodable,
{
    let req: TagRequest = req.into();
    let mr_request = MessageRequest::new(0x4D, req.tag, value);
    let resp = client.send(mr_request).await?;
    if resp.reply_service != 0xCD {
        return Err(rseip_core::Error::<InnerError>::from_invalid_data()
            .with_context(format!(
                "unexpected reply service for write tag service: {:#0x}",
                resp.reply_service
            ))
            .into());
    }
    if resp.status.general != 0 {
        return Err(CipError::Cip(resp.status).into());
    }
    Ok(())
}

/// Read Tag Fragmented Service
async fn ab_read_tag_fragmented<C: MessageService<Error = Error>, F, R>(
    client: &mut C,
    req: ReadFragmentedRequest<F, R>,
) -> Result<(bool, R)>
where
    F: Fn(u16, Bytes) -> Result<R>,
{
    assert!(req.total > req.offset);
    let ReadFragmentedRequest {
        tag,
        total,
        offset,
        decoder,
    } = req;
    let mr_request = MessageRequest::new(
        0x52,
        tag,
        LazyEncode {
            f: |buf: &mut BytesMut| {
                buf.put_u16_le(total);
                buf.put_u16_le(offset);
                buf.put_u16_le(0);
                Ok(())
            },
            bytes_count: 6,
        },
    );
    let resp = client.send(mr_request).await?;
    if resp.reply_service != 0xD2 {
        return Err(rseip_core::Error::<InnerError>::from_invalid_data()
            .with_context(format!(
                "unexpected reply service for read tag fragmented service: {:#0x}",
                resp.reply_service
            ))
            .into());
    }
    if resp.status.general != 0 && resp.status.general != 0x06 {
        return Err(CipError::Cip(resp.status).into());
    }
    let data = resp.data;
    assert!(data.len() >= 4);
    let tag_type = LittleEndian::read_u16(&data[0..2]);
    let data = (decoder)(tag_type, data.slice(2..))?;
    Ok((resp.status.general == 0x06, data))
}

/// Write Tag Fragmented Service, enables client applications to write to a tag
/// in the controller whose data will not fit into a single packet (approximately 500 bytes)
async fn ab_write_tag_fragmented<C: MessageService<Error = Error>, D: Encodable>(
    client: &mut C,
    req: WriteFragmentedRequest<D>,
) -> Result<bool> {
    assert!(req.total > req.offset);
    let WriteFragmentedRequest {
        tag,
        tag_type,
        total,
        offset,
        data,
    } = req;
    let mr_request = MessageRequest::new(
        0x53,
        tag,
        LazyEncode {
            f: |buf: &mut BytesMut| {
                buf.put_u16_le(tag_type);
                buf.put_u16_le(total);
                buf.put_u16_le(offset);
                buf.put_u16_le(0);
                data.encode(buf)?;
                Ok(())
            },
            bytes_count: 6,
        },
    );
    let resp = client.send(mr_request).await?;
    if resp.reply_service != 0xD3 {
        return Err(rseip_core::Error::<InnerError>::from_invalid_data()
            .with_context(format!(
                "unexpected reply for write tag fragmented service: {:#0x}",
                resp.reply_service
            ))
            .into());
    }
    if resp.status.general != 0 && resp.status.general != 0x06 {
        return Err(CipError::Cip(resp.status).into());
    }

    Ok(resp.status.general == 0x06)
}

/// Read Modify Write Tag Service, modifies Tag data with individual bit resolution
async fn ab_read_modify_write<C: MessageService<Error = Error>, const N: usize>(
    client: &mut C,
    req: ReadModifyWriteRequest<N>,
) -> Result<()> {
    let ReadModifyWriteRequest {
        tag,
        or_mask,
        and_mask,
    } = req;
    let mr_request = MessageRequest::new(
        0x4E,
        tag,
        LazyEncode {
            f: |buf: &mut BytesMut| {
                buf.put_u16_le(N as u16);
                buf.put_slice(&or_mask);
                buf.put_slice(&and_mask);
                Ok(())
            },
            bytes_count: 6,
        },
    );
    let resp = client.send(mr_request).await?;
    if resp.reply_service != 0xCE {
        return Err(rseip_core::Error::<InnerError>::from_invalid_data()
            .with_context(format!(
                "unexpected reply service for read modify tag service: {:#0x}",
                resp.reply_service
            ))
            .into());
    }
    if resp.status.general != 0 {
        return Err(CipError::Cip(resp.status).into());
    }

    Ok(())
}

/// N: only 1,2,4,8,12 accepted
pub struct ReadModifyWriteRequest<const N: usize> {
    tag: EPath,
    or_mask: [u8; N],
    and_mask: [u8; N],
}

impl<const N: usize> ReadModifyWriteRequest<N> {
    pub fn new() -> Self {
        assert!(N == 1 || N == 2 || N == 4 || N == 8 || N == 12);
        Self {
            tag: Default::default(),
            or_mask: [0; N],
            and_mask: [0xF; N],
        }
    }

    pub fn tag(mut self, val: impl Into<EPath>) -> Self {
        self.tag = val.into();
        self
    }

    /// 1 mask sets bit to 1
    pub fn or_mask(mut self, mask: impl Into<[u8; N]>) -> Self {
        self.or_mask = mask.into();
        self
    }

    pub fn or_mask_mut(&mut self) -> &mut [u8] {
        &mut self.or_mask
    }

    /// 0 mask resets bit to 0
    pub fn and_mask(mut self, mask: impl Into<[u8; N]>) -> Self {
        self.and_mask = mask.into();
        self
    }

    pub fn and_mask_mut(&mut self) -> &mut [u8] {
        &mut self.and_mask
    }
}

pub struct WriteFragmentedRequest<D> {
    tag: EPath,
    tag_type: u16,
    total: u16,
    offset: u16,
    data: D,
}

impl<D> WriteFragmentedRequest<D> {
    pub fn new(total: u16, data: D) -> Self {
        Self {
            tag: Default::default(),
            tag_type: 0,
            total,
            offset: 0,
            data,
        }
    }

    pub fn tag(mut self, val: impl Into<EPath>) -> Self {
        self.tag = val.into();
        self
    }

    pub fn tag_type(mut self, val: u16) -> Self {
        self.tag_type = val;
        self
    }

    pub fn total(mut self, val: u16) -> Self {
        self.total = val;
        self
    }

    pub fn offset(mut self, val: u16) -> Self {
        self.offset = val;
        self
    }

    pub fn data(mut self, data: D) -> Self {
        self.data = data;
        self
    }
}

pub struct ReadFragmentedRequest<F, R>
where
    F: Fn(u16, Bytes) -> Result<R>,
{
    tag: EPath,
    total: u16,
    offset: u16,
    decoder: F,
}

impl<F, R> ReadFragmentedRequest<F, R>
where
    F: Fn(u16, Bytes) -> Result<R>,
{
    pub fn new(total: u16, f: F) -> Self {
        Self {
            tag: Default::default(),
            total,
            offset: 0,
            decoder: f,
        }
    }

    pub fn tag(mut self, val: impl Into<EPath>) -> Self {
        self.tag = val.into();
        self
    }

    pub fn total(mut self, val: u16) -> Self {
        self.total = val;
        self
    }

    pub fn offset(mut self, val: u16) -> Self {
        self.offset = val;
        self
    }

    pub fn decoder(mut self, f: F) -> Self {
        self.decoder = f;
        self
    }
}

pub struct TagRequest {
    tag: EPath,
    count: u16,
}

impl From<EPath> for TagRequest {
    #[inline]
    fn from(tag: EPath) -> Self {
        Self { tag, count: 1 }
    }
}

impl From<(EPath, u16)> for TagRequest {
    #[inline]
    fn from(src: (EPath, u16)) -> Self {
        Self {
            tag: src.0,
            count: src.1,
        }
    }
}

pub struct ElementCount(pub u16);

impl Encodable for ElementCount {
    #[inline]
    fn encode(self, dst: &mut BytesMut) -> cip::Result<()> {
        dst.put_u16_le(self.0);
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        2
    }
}
