//! TODO this needs to be merged with fate-rs

extern crate num_traits;
use self::num_traits::Unsigned;

/// An unsigned width+height pair.
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Extent2<T: Unsigned> {
    /// Width.
    pub w: T,
    /// Height.
    pub h: T,
}

impl<T: Unsigned> From<(T,T)> for Extent2<T> {
    fn from(pair: (T,T)) -> Self {
        let (w, h) = pair;
        Self { w, h }
    }
}

impl<T: Unsigned> Extent2<T> {
    /// Quickly create an `Extent2`.
    pub fn new(w: T, h: T) -> Self {
        Self::from((w, h))
    }
}

/// XXX This type was already defined in FATE
pub type Rgba32 = u32;

// XXX Already defined
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Vec2<T> { pub x: T, pub y: T, }


