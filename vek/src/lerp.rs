//! NOTE: If you want to lerp with integer types, convert them to floats first, lerp on them, then
//! convert them back to integers. This eases our implementation and gives you explicit control
//! over the conversion behavior.


// TODO would be good to be able to directly Lerp on Rgb<T: ColorChannel> though

use core::ops::*;

use vec::*;
use clamp::{Clamp01, clamp01};

// TODO split into two traits, Lerp and LerpUnclamped

pub trait Lerp<Progress=f32>: Sized + Add<Output=Self> + Mul<Progress, Output=Self>
    where Progress : Clone + Clamp01 + Sub<Output=Progress>
{
    fn lerp_unclamped(from: Self, to: Self, progress: Progress) -> Self {
        let progress_dup = progress.clone();
        from*(Progress::one()-progress) + to*progress_dup
    }
    fn lerp(from: Self, to: Self, progress: Progress) -> Self {
        Self::lerp_unclamped(from, to, clamp01(progress))
    }
}

pub fn lerp_unclamped<Progress, T>(from: T, to: T, progress: Progress) -> T 
    where T: Lerp<Progress>, Progress : Clone + Clamp01 + Sub<Output=Progress>
{
    T::lerp_unclamped(from, to, progress)
}
pub fn lerp<Progress, T>(from: T, to: T, progress: Progress) -> T
    where T: Lerp<Progress>, Progress : Clone + Clamp01 + Sub<Output=Progress>
{
    T::lerp(from, to, progress)
}

impl Lerp<f32> for f32 {}
impl Lerp<f64> for f64 {}

macro_rules! lerp_impl_for_vecs {
    ($($Vec:ident)+) => {
        $(
            impl<T: Clone + Clamp01 + Sub<Output=T>> Lerp<     T > for $Vec <T> {}
            // TODO: Extend Clamp to vectors somehow
            // impl<T: Clone + Clamp01 + Sub<Output=T>> Lerp<$Vec<T>> for $Vec <T> {}
        )+
    }
}

lerp_impl_for_vecs!(Vec4 Vec3 Vec2 Xyzw Xyz Xy Rgba Rgb Uvw Uv Extent3 Extent2);
