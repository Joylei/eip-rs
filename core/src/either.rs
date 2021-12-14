// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> Either<A, B> {
    #[inline]
    pub fn left(&self) -> Option<&A> {
        match self {
            Self::Left(ref v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn left_mut(&mut self) -> Option<&mut A> {
        match self {
            Self::Left(ref mut v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn into_left(self) -> Option<A> {
        match self {
            Self::Left(v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn right(&self) -> Option<&B> {
        match self {
            Self::Right(ref v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn right_mut(&mut self) -> Option<&mut B> {
        match self {
            Self::Right(ref mut v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn into_right(self) -> Option<B> {
        match self {
            Self::Right(v) => Some(v),
            _ => None,
        }
    }
}
