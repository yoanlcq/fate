//! NOTE: If you want to lerp with integer types, convert them to floats first, lerp on them, then
//! convert them back to integers. This eases our implementation and gives you explicit control
//! over the conversion behavior.


// TODO would be good to be able to directly Lerp on Rgb<T: ColorChannel> though

use core::ops::*;

use vec::*;
use clamp::{Clamp01, clamp01};

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

impl Lerp<f32> for         f32  {}
impl Lerp<f32> for Vec2   <f32> {}
impl Lerp<f32> for Vec3   <f32> {}
impl Lerp<f32> for Vec4   <f32> {}
impl Lerp<f32> for Xyzw   <f32> {}
impl Lerp<f32> for Xyz    <f32> {}
impl Lerp<f32> for Xy     <f32> {}
impl Lerp<f32> for Uvw    <f32> {}
impl Lerp<f32> for Uv     <f32> {}
impl Lerp<f32> for Rgba   <f32> {}
impl Lerp<f32> for Rgb    <f32> {}
impl Lerp<f32> for Extent3<f32> {}
impl Lerp<f32> for Extent2<f32> {}
impl Lerp<f64> for         f64  {}
impl Lerp<f64> for Vec2   <f64> {}
impl Lerp<f64> for Vec3   <f64> {}
impl Lerp<f64> for Vec4   <f64> {}
impl Lerp<f64> for Xyzw   <f64> {}
impl Lerp<f64> for Xyz    <f64> {}
impl Lerp<f64> for Xy     <f64> {}
impl Lerp<f64> for Uvw    <f64> {}
impl Lerp<f64> for Uv     <f64> {}
impl Lerp<f64> for Rgba   <f64> {}
impl Lerp<f64> for Rgb    <f64> {}
impl Lerp<f64> for Extent3<f64> {}
impl Lerp<f64> for Extent2<f64> {}
