// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use bytes::{BufMut, Bytes, BytesMut};
use rseip_core::String;
use std::ops::{Deref, DerefMut};

/// EPATH for unconnected send
pub const EPATH_CONNECTION_MANAGER: &'static [u8] = &[0x20, 0x06, 0x24, 0x01];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    Symbol(String),
    Class(u16),
    Instance(u16),
    Attribute(u16),
    Element(u32),
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EPath(Vec<Segment>);

impl EPath {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn into_vec(self) -> Vec<Segment> {
        self.0
    }

    #[inline]
    pub fn into_iter(self) -> impl IntoIterator<Item = Segment> {
        self.0.into_iter()
    }

    #[inline]
    pub fn with_class(mut self, class_id: u16) -> Self {
        self.0.push(Segment::Class(class_id));
        self
    }

    #[inline]
    pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.0.push(Segment::Symbol(symbol.into()));
        self
    }

    #[inline]
    pub fn with_instance(mut self, instance_id: u16) -> Self {
        self.0.push(Segment::Instance(instance_id));
        self
    }

    #[inline]
    pub fn with_element(mut self, element_idx: u32) -> Self {
        self.0.push(Segment::Element(element_idx));
        self
    }

    /// with port & default slot 0
    #[inline]
    pub fn with_port(mut self, port: u16) -> Self {
        self.0.push(Segment::Port(PortSegment {
            port,
            link: Bytes::from_static(&[0]),
        }));
        self
    }

    /// with port & slot
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
    #[inline]
    fn from(src: Vec<Segment>) -> Self {
        Self(src)
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
    #[inline]
    fn default() -> Self {
        Self {
            port: 1,                        // Backplane
            link: Bytes::from_static(&[0]), // slot 0
        }
    }
}
