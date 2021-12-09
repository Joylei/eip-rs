// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::symbol::GetInstanceAttributeList;
use super::*;
use crate::client::ab_eip::interceptor::HasMoreInterceptor;
use crate::StdResult;
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, BytesMut};
use rseip_core::codec::{BytesHolder, Encode, Encoder};

/// AB related operations
#[async_trait::async_trait(?Send)]
pub trait AbService {
    /// Read Tag Service,
    /// CIP Data Table Read
    async fn read_tag<'de, P, R>(&mut self, req: P) -> Result<R>
    where
        P: Into<TagRequest>,
        R: Decode<'de> + 'static;

    /// Write Tag Service,
    /// CIP Data Table Write
    async fn write_tag<D>(&mut self, tag: EPath, value: D) -> Result<()>
    where
        D: Encode;

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
    async fn write_tag_fragmented<D: Encode>(
        &mut self,
        req: WriteFragmentedRequest<D>,
    ) -> Result<bool>;

    /// Read Modify Write Tag Service, modifies Tag data with individual bit resolution
    async fn read_modify_write<const N: usize>(
        &mut self,
        req: ReadModifyWriteRequest<N>,
    ) -> Result<()>;

    fn list_tag(&mut self) -> GetInstanceAttributeList<Self>
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
            async fn read_tag<'de, P, R>(&mut self, req: P) -> Result<R>
            where
                P: Into<TagRequest>,
                R: Decode<'de> + 'static,
            {
                let res = ab_read_tag(self, req).await?;
                Ok(res)
            }

            /// Write Tag Service,
            /// CIP Data Table Write
            #[inline]
            async fn write_tag<D>(&mut self, tag: EPath, value: D) -> Result<()>
            where
                D: Encode,
            {
                ab_write_tag(self, tag, value).await?;
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
            async fn write_tag_fragmented<D: Encode>(
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
            fn list_tag(&mut self) -> GetInstanceAttributeList<Self>
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
async fn ab_read_tag<'de, C, P, R>(client: &mut C, req: P) -> Result<R>
where
    C: MessageService<Error = ClientError>,
    P: Into<TagRequest>,
    R: Decode<'de> + 'static,
{
    let req: TagRequest = req.into();
    let mr = MessageRequest::new(SERVICE_READ_TAG, req.tag, req.count);
    let resp: MessageReply<_> = client.send(mr).await?;
    resp.expect_service::<ClientError>(SERVICE_READ_TAG + REPLY_MASK)?;
    Ok(resp.data)
}

/// Write Tag Service,
/// CIP Data Table Write
async fn ab_write_tag<C, D>(client: &mut C, tag: EPath, value: D) -> Result<()>
where
    C: MessageService<Error = ClientError>,
    D: Encode,
{
    let mr = MessageRequest::new(SERVICE_WRITE_TAG, tag, value);
    let resp: MessageReply<()> = client.send(mr).await?;
    resp.expect_service::<ClientError>(SERVICE_WRITE_TAG + REPLY_MASK)?;
    Ok(())
}

/// Read Tag Fragmented Service
async fn ab_read_tag_fragmented<C, F, R>(
    client: &mut C,
    req: ReadFragmentedRequest<F, R>,
) -> Result<(bool, R)>
where
    C: MessageService<Error = ClientError>,
    F: Fn(u16, Bytes) -> Result<R>,
{
    debug_assert!(req.total > req.offset);
    let ReadFragmentedRequest {
        tag,
        total,
        offset,
        decoder,
    } = req;

    let mr = MessageRequest::new(SERVICE_READ_TAG_FRAGMENTED, tag, [total, offset, 0]);
    let resp: HasMoreInterceptor<BytesHolder> = client.send(mr).await?;
    resp.0
        .expect_service::<ClientError>(SERVICE_READ_TAG_FRAGMENTED + REPLY_MASK)?;
    let data: Bytes = resp.0.data.into();
    assert!(data.len() >= 4);
    let tag_type = LittleEndian::read_u16(&data[0..2]);
    let data = (decoder)(tag_type, data.slice(2..))?;
    Ok((resp.0.status.has_more(), data))
}

/// Write Tag Fragmented Service, enables client applications to write to a tag
/// in the controller whose data will not fit into a single packet (approximately 500 bytes)
async fn ab_write_tag_fragmented<C, D>(
    client: &mut C,
    req: WriteFragmentedRequest<D>,
) -> Result<bool>
where
    C: MessageService<Error = ClientError>,
    D: Encode,
{
    debug_assert!(req.total > req.offset);

    struct DataHolder<D> {
        tag_type: u16,
        total: u16,
        offset: u16,
        data: D,
    }

    impl<D: Encode> Encode for DataHolder<D> {
        #[inline]
        fn encode<A: Encoder>(self, buf: &mut BytesMut, encoder: &mut A) -> StdResult<(), A::Error>
        where
            Self: Sized,
        {
            buf.put_u16_le(self.tag_type);
            buf.put_u16_le(self.total);
            buf.put_u16_le(self.offset);
            buf.put_u16_le(0);
            self.data.encode(buf, encoder)?;
            Ok(())
        }
        #[inline]
        fn encode_by_ref<A: Encoder>(
            &self,
            buf: &mut BytesMut,
            encoder: &mut A,
        ) -> StdResult<(), A::Error> {
            buf.put_u16_le(self.tag_type);
            buf.put_u16_le(self.total);
            buf.put_u16_le(self.offset);
            buf.put_u16_le(0);
            self.data.encode_by_ref(buf, encoder)?;
            Ok(())
        }
        #[inline]
        fn bytes_count(&self) -> usize {
            8 + self.data.bytes_count()
        }
    }

    let WriteFragmentedRequest {
        tag,
        tag_type,
        total,
        offset,
        data,
    } = req;
    let mr = MessageRequest::new(
        SERVICE_WRITE_TAG_FRAGMENTED,
        tag,
        DataHolder {
            tag_type,
            total,
            offset,
            data,
        },
    );
    let resp: HasMoreInterceptor<()> = client.send(mr).await?;
    resp.expect_service::<ClientError>(SERVICE_WRITE_TAG_FRAGMENTED + REPLY_MASK)?;
    Ok(resp.0.status.has_more())
}

/// Read Modify Write Tag Service, modifies Tag data with individual bit resolution
async fn ab_read_modify_write<C, const N: usize>(
    client: &mut C,
    req: ReadModifyWriteRequest<N>,
) -> Result<()>
where
    C: MessageService<Error = ClientError>,
{
    struct DataHolder<const N: usize> {
        or_mask: [u8; N],
        and_mask: [u8; N],
    }
    impl<const N: usize> Encode for DataHolder<N> {
        #[inline]
        fn encode_by_ref<A: Encoder>(
            &self,
            buf: &mut BytesMut,
            _encoder: &mut A,
        ) -> StdResult<(), A::Error> {
            buf.put_u16_le(N as u16);
            buf.put_slice(&self.or_mask);
            buf.put_slice(&self.and_mask);
            Ok(())
        }

        #[inline]
        fn bytes_count(&self) -> usize {
            2 + N * 2
        }
    }

    let ReadModifyWriteRequest {
        tag,
        or_mask,
        and_mask,
    } = req;

    let mr_request = MessageRequest::new(
        SERVICE_READ_MODIFY_WRITE_TAG,
        tag,
        DataHolder { and_mask, or_mask },
    );
    let resp: MessageReply<()> = client.send(mr_request).await?;
    resp.expect_service::<ClientError>(SERVICE_READ_MODIFY_WRITE_TAG + REPLY_MASK)?;
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
            and_mask: [0xFF; N],
        }
    }

    pub fn tag(mut self, val: impl Into<EPath>) -> Self {
        self.tag = val.into();
        self
    }

    /// Array of OR modify masks; 1 mask sets bit to 1
    pub fn or_mask(mut self, mask: impl Into<[u8; N]>) -> Self {
        self.or_mask = mask.into();
        self
    }

    /// Array of OR modify masks; 1 mask sets bit to 1
    pub fn or_mask_mut(&mut self) -> &mut [u8] {
        &mut self.or_mask
    }

    /// Array of AND modify masks; 0 mask resets bit to 0
    pub fn and_mask(mut self, mask: impl Into<[u8; N]>) -> Self {
        self.and_mask = mask.into();
        self
    }

    /// Array of AND modify masks; 0 mask resets bit to 0
    pub fn and_mask_mut(&mut self) -> &mut [u8] {
        &mut self.and_mask
    }
}

impl<const N: usize> Default for ReadModifyWriteRequest<N> {
    fn default() -> Self {
        Self::new()
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
