use std::ops::{Deref, DerefMut};

use bytes::{BufMut, Bytes, BytesMut};

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
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn into_vec(self) -> Vec<Segment> {
        self.0
    }
}

impl Deref for EPath {
    type Target = [Segment];
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EPath {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<Segment>> for EPath {
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
    pub link: u32,
}

impl Default for PortSegment {
    #[inline(always)]
    fn default() -> Self {
        Self { port: 1, link: 0 }
    }
}
