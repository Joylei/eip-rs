// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use bytes::{BufMut, Bytes, BytesMut};
use core::ops::{Deref, DerefMut};
use rseip_core::String;
use smallvec::{smallvec, SmallVec};

/// EPATH for unconnected send
pub const EPATH_CONNECTION_MANAGER: &[u8] = &[0x20, 0x06, 0x24, 0x01];

/// Segment of EPATH
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// symbolic, ANSI Ext. String
    Symbol(String),
    /// class id
    Class(u16),
    /// instance id
    Instance(u16),
    /// attribute id
    Attribute(u16),
    /// element index
    Element(u32),
    /// port segment
    Port(PortSegment),
}

impl Segment {
    /// is port segment?
    #[inline]
    pub fn is_port(&self) -> bool {
        match self {
            Self::Port(_) => true,
            _ => false,
        }
    }
}

type Array = [Segment; 4];

/// EPATH
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EPath(SmallVec<Array>);

impl EPath {
    /// new object
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// into inner
    pub fn into_inner(self) -> SmallVec<Array> {
        self.0
    }

    /// append class id
    #[inline]
    pub fn with_class(mut self, class_id: u16) -> Self {
        self.0.push(Segment::Class(class_id));
        self
    }

    /// append symbolic
    #[inline]
    pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.0.push(Segment::Symbol(symbol.into()));
        self
    }

    /// append instance id
    #[inline]
    pub fn with_instance(mut self, instance_id: u16) -> Self {
        self.0.push(Segment::Instance(instance_id));
        self
    }

    /// append element id
    #[inline]
    pub fn with_element(mut self, element_idx: u32) -> Self {
        self.0.push(Segment::Element(element_idx));
        self
    }

    /// append port & default slot 0
    #[inline]
    pub fn with_port(mut self, port: u16) -> Self {
        self.0.push(Segment::Port(PortSegment {
            port,
            link: Bytes::from_static(&[0]),
        }));
        self
    }

    /// append port & slot
    #[inline]
    pub fn with_port_slot(mut self, port: u16, slot: u8) -> Self {
        let mut buf = BytesMut::new();
        buf.put_u8(slot);
        self.0.push(Segment::Port(PortSegment {
            port,
            link: buf.freeze(),
        }));
        self
    }

    /// from symbolic segment, ANSI Ext. String
    ///
    /// ## Note
    /// No validation applied
    #[inline]
    pub fn from_symbol(symbol: impl Into<String>) -> Self {
        EPath(smallvec![Segment::Symbol(symbol.into())])
    }

    #[inline]
    pub fn insert(&mut self, index: usize, item: Segment) {
        self.0.insert(index, item)
    }

    #[inline]
    pub fn push(&mut self, item: Segment) {
        self.0.push(item);
    }

    #[inline]
    pub fn remove(&mut self, index: usize) {
        self.0.remove(index);
    }
}

impl IntoIterator for EPath {
    type Item = Segment;
    type IntoIter = rseip_core::iter::IntoIter<Array>;
    fn into_iter(self) -> Self::IntoIter {
        rseip_core::iter::IntoIter::new(self.0)
    }
}

impl Deref for EPath {
    type Target = [Segment];
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EPath {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<Segment>> for EPath {
    fn from(src: Vec<Segment>) -> Self {
        Self(SmallVec::from_vec(src))
    }
}

/// EPATH Port Segment
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PortSegment {
    /// Port to leave Current Node (1 if Backplane)
    pub port: u16,
    /// link address to route packet (number or IP address)
    pub link: Bytes,
}

impl Default for PortSegment {
    fn default() -> Self {
        Self {
            port: 1,                        // Backplane
            link: Bytes::from_static(&[0]), // slot 0
        }
    }
}
