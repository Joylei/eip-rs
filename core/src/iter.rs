// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use smallvec::{Array, SmallVec};

/// smallvec IntoIter proxy
pub struct IntoIter<A: Array>(smallvec::IntoIter<A>);

impl<A: Array> IntoIter<A> {
    #[inline]
    pub fn new(vec: SmallVec<A>) -> Self {
        Self(vec.into_iter())
    }
}

impl<A: Array> Iterator for IntoIter<A> {
    type Item = A::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
