extern crate num_traits;

use self::num_traits::{Zero, One};

pub trait PartialMinMax: PartialOrd + Sized {
    fn partial_min(v1: Self, v2: Self) -> Self {
        if v1 < v2 { v1 } else { v2 }
    }
    fn partial_max(v1: Self, v2: Self) -> Self {
        if v1 > v2 { v1 } else { v2 }
    }
}

pub fn partial_max<T: PartialMinMax>(v1: T, v2: T) -> T {
    T::partial_max(v1, v2)
}
pub fn partial_min<T: PartialMinMax>(v1: T, v2: T) -> T {
    T::partial_min(v1, v2)
}

pub trait Clamp: PartialMinMax {
    fn clamped(self, lower: Self, upper: Self) -> Self {
        Self::partial_min(Self::partial_max(self, lower), upper)
    }
    fn is_between(self, lower: Self, upper: Self) -> bool {
        lower <= self && self <= upper
    }
}
pub trait Clamp01: Zero + One + Clamp {
    fn clamped01(self) -> Self {
        self.clamped(Self::zero(), Self::one())
    }
    fn is_between01(self) -> bool {
        self.is_between(Self::zero(), Self::one())
    }
}
pub fn clamp<T: Clamp>(x: T, lower: T, upper: T) -> T {
    x.clamped(lower, upper)
}
pub fn clamp01<T: Clamp01>(x: T) -> T {
    x.clamped01()
}

impl PartialMinMax for f32 {}
impl PartialMinMax for f64 {}
impl PartialMinMax for i8  {}
impl PartialMinMax for i16 {}
impl PartialMinMax for i32 {}
impl PartialMinMax for i64 {}
impl PartialMinMax for u8  {}
impl PartialMinMax for u16 {}
impl PartialMinMax for u32 {}
impl PartialMinMax for u64 {}

impl Clamp for f32 {}
impl Clamp for f64 {}
impl Clamp for i8  {}
impl Clamp for i16 {}
impl Clamp for i32 {}
impl Clamp for i64 {}
impl Clamp for u8  {}
impl Clamp for u16 {}
impl Clamp for u32 {}
impl Clamp for u64 {}

impl Clamp01 for f32 {}
impl Clamp01 for f64 {}
impl Clamp01 for i8  {}
impl Clamp01 for i16 {}
impl Clamp01 for i32 {}
impl Clamp01 for i64 {}
impl Clamp01 for u8  {}
impl Clamp01 for u16 {}
impl Clamp01 for u32 {}
impl Clamp01 for u64 {}
