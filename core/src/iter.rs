use smallvec::{Array, SmallVec};

/// smallvec IntoIter proxy
pub struct IntoIter<A: Array>(smallvec::IntoIter<A>);

impl<A: Array> IntoIter<A> {
    pub fn new(vec: SmallVec<A>) -> Self {
        Self(vec.into_iter())
    }
}

impl<A: Array> Iterator for IntoIter<A> {
    type Item = A::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
