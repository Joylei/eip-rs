// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::*;
use crate::{codec::encode::LazyEncode, eip::context::EipContext};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, BytesMut};
use futures_util::future::BoxFuture;
use std::{convert::TryFrom, io, mem, net::SocketAddrV4};
use tokio::net::TcpStream;

/// AB EIP Client
pub type AbEipClient = Client<AbEipDriver>;

/// AB EIP Connection
pub type AbEipConnection = Connection<AbEipDriver>;

/// AB EIP driver
pub struct AbEipDriver;

impl Driver for AbEipDriver {
    type Endpoint = SocketAddrV4;
    type Service = EipContext<TcpStream>;

    #[inline]
    fn build_service(addr: &Self::Endpoint) -> BoxFuture<Result<Self::Service>> {
        EipDriver::build_service(addr)
    }
}

impl AbEipClient {
    pub async fn new_host_lookup(host: impl AsRef<str>) -> io::Result<Self> {
        let addr = resolve_host(host).await?;
        Ok(Self::new(addr))
    }
}

impl AbEipConnection {
    pub async fn new_host_lookup(host: impl AsRef<str>, options: Options) -> io::Result<Self> {
        let addr = resolve_host(host).await?;
        Ok(Self::new(addr, options))
    }
}

#[async_trait::async_trait(?Send)]
pub trait AbService {
    /// Read Tag Service,
    /// CIP Data Table Read
    async fn read_tag<P, R>(&mut self, req: P) -> Result<R>
    where
        P: Into<TagRequest>,
        R: TryFrom<Bytes>,
        R::Error: Into<crate::Error> + std::error::Error;

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
                R::Error: Into<crate::Error> + std::error::Error,
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
        }
    };
}

impl_service!(AbEipClient);
impl_service!(AbEipConnection);
impl_service!(MaybeConnected<AbEipDriver>);

/// Read Tag Service,
/// CIP Data Table Read
async fn ab_read_tag<C: MessageRouter, P, R>(client: &mut C, req: P) -> Result<R>
where
    P: Into<TagRequest>,
    R: TryFrom<Bytes>,
    R::Error: Into<crate::Error> + std::error::Error,
{
    let req: TagRequest = req.into();
    let mr_request = MessageRouterRequest::new(0x4C, req.tag, ElementCount(req.count));
    let resp = client.send(mr_request).await?;
    if resp.reply_service != 0xCC {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "unexpected reply for read tag service",
        )
        .into());
    }
    R::try_from(resp.data).map_err(|e| e.into())
}

/// Write Tag Service,
/// CIP Data Table Write
async fn ab_write_tag<C: MessageRouter, P, D>(
    client: &mut C,
    req: P,
    value: TagValue<D>,
) -> Result<()>
where
    P: Into<TagRequest>,
    D: Encodable,
{
    let req: TagRequest = req.into();
    let mr_request = MessageRouterRequest::new(0x4D, req.tag, value);
    let resp = client.send(mr_request).await?;
    if resp.reply_service != 0xCD {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "unexpected reply for write tag service",
        )
        .into());
    }
    Ok(())
}

/// Read Tag Fragmented Service
async fn ab_read_tag_fragmented<C: MessageRouter, F, R>(
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
    let mr_request = MessageRouterRequest::new(
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
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "unexpected reply for read tag service",
        )
        .into());
    }
    if resp.status.general != 0 && resp.status.general != 0x06 {
        return Err(io::Error::new(io::ErrorKind::Other, "bad status").into());
    }
    let data = resp.data;
    assert!(data.len() >= 4);
    let tag_type = LittleEndian::read_u16(&data[0..2]);
    let data = (decoder)(tag_type, data.slice(2..))?;
    Ok((resp.status.general == 0x06, data))
}

/// Write Tag Fragmented Service, enables client applications to write to a tag
/// in the controller whose data will not fit into a single packet (approximately 500 bytes)
async fn ab_write_tag_fragmented<C: MessageRouter, D: Encodable>(
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
    let mr_request = MessageRouterRequest::new(
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
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "unexpected reply for read tag service",
        )
        .into());
    }
    if resp.status.general != 0 && resp.status.general != 0x06 {
        return Err(io::Error::new(io::ErrorKind::Other, "bad status").into());
    }

    Ok(resp.status.general == 0x06)
}

/// Read Modify Write Tag Service, modifies Tag data with individual bit resolution
async fn ab_read_modify_write<C: MessageRouter, const N: usize>(
    client: &mut C,
    req: ReadModifyWriteRequest<N>,
) -> Result<()> {
    let ReadModifyWriteRequest {
        tag,
        or_mask,
        and_mask,
    } = req;
    let mr_request = MessageRouterRequest::new(
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
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "unexpected reply for read tag service",
        )
        .into());
    }
    if resp.status.general != 0 {
        return Err(io::Error::new(io::ErrorKind::Other, "bad status").into());
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

#[derive(Debug)]
pub enum TagValue<D = Bytes> {
    /// atomic data type: BOOL
    BOOL(bool),
    /// atomic data type: DWORD, 32-bit boolean array
    DWORD(u32),
    /// atomic data type: SINT, 8-bit integer
    SINT(i8),
    /// atomic data type: INT, 16-bit integer
    INT(i16),
    /// atomic data type: DINT, 32-bit integer
    DINT(i32),
    /// atomic data type: LINT, 64-bit integer
    LINT(i64),
    /// atomic data type: REAL, 32-bit float
    REAL(f32),
    UDT(D),
}

impl TryFrom<Bytes> for TagValue<Bytes> {
    type Error = Error;
    #[inline]
    fn try_from(src: Bytes) -> Result<Self> {
        //TODO: verify len
        assert!(src.len() >= 4);
        let tag_type = LittleEndian::read_u16(&src[0..2]);
        let val = match tag_type {
            0xC2 => TagValue::SINT(unsafe { mem::transmute(src[2]) }),
            0xC3 => TagValue::INT(LittleEndian::read_i16(&src[2..4])),
            0xC4 => {
                assert!(src.len() >= 6);
                TagValue::DINT(LittleEndian::read_i32(&src[2..6]))
            }
            0xCA => {
                assert!(src.len() >= 6);
                TagValue::REAL(LittleEndian::read_f32(&src[2..6]))
            }
            0xD3 => {
                assert!(src.len() >= 6);
                TagValue::DWORD(LittleEndian::read_u32(&src[2..6]))
            }
            0xC5 => {
                assert!(src.len() >= 10);
                TagValue::LINT(LittleEndian::read_i64(&src[2..10]))
            }
            0xC1 => TagValue::BOOL(src[4] == 255),
            _ => TagValue::UDT(src),
        };
        Ok(val)
    }
}

impl<D: Encodable> Encodable for TagValue<D> {
    #[inline]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        match self {
            Self::BOOL(v) => {
                dst.put_u16_le(0xC1);
                dst.put_slice(&[1, 0]);
                dst.put_u8(if v { 255 } else { 0 });
                dst.put_u8(0);
            }
            Self::SINT(v) => {
                dst.put_u16_le(0xC2);
                dst.put_slice(&[1, 0]);
                dst.put_i8(v);
                dst.put_u8(0);
            }
            Self::INT(v) => {
                dst.put_u16_le(0xC3);
                dst.put_slice(&[1, 0]);
                dst.put_i16_le(v);
            }
            Self::DINT(v) => {
                dst.put_u16_le(0xC4);
                dst.put_slice(&[1, 0]);
                dst.put_i32_le(v);
            }
            Self::REAL(v) => {
                dst.put_u16_le(0xCA);
                dst.put_slice(&[1, 0]);
                dst.put_f32_le(v);
            }
            Self::DWORD(v) => {
                dst.put_u16_le(0xD3);
                dst.put_slice(&[1, 0]);
                dst.put_u32_le(v);
            }
            Self::LINT(v) => {
                dst.put_u16_le(0xC5);
                dst.put_slice(&[1, 0]);
                dst.put_i64_le(v);
            }
            Self::UDT(data) => {
                data.encode(dst)?;
            }
        };
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        match self {
            Self::BOOL(_) => 6,
            Self::SINT(_) => 6,
            Self::INT(_) => 6,
            Self::DINT(_) => 8,
            Self::REAL(_) => 8,
            Self::DWORD(_) => 8,
            Self::LINT(_) => 12,
            Self::UDT(v) => v.bytes_count(),
        }
    }
}

struct ElementCount(u16);

impl Encodable for ElementCount {
    #[inline]
    fn encode(self, dst: &mut BytesMut) -> Result<()> {
        dst.put_u16_le(self.0);
        Ok(())
    }

    #[inline]
    fn bytes_count(&self) -> usize {
        2
    }
}
