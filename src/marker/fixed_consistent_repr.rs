//! The `FixedConsistentSize` and `FixedConsistentRepr` marker traits.
//!
//! See these traits' doc for explanations on their meaning.  
//! *Note: this module's name is not shortened to `fixed` because it might be confused with a module
//!  for fixed-point number data types.*
//! 
//! As for `FixedConsistentSize`'s use cases : it is sometimes good to guarantee that some of your data
//! types have the exact same size across targets.  
//!
//! Is is true that, more often than not, data structures should be tuned for the target
//! hardware, with regards to cache size, CPU arch-specific instructions, etc.  
//! However, there are cases where that reasoning is flawed : Why would your player's score be an 
//! `usize` when you know that it stores either 32 or 64 bits worth of integer data, depending on
//! the target ? You would basically be forbidding anyone running in 32-bit mode to beat Bob who
//! earned the title of world champion with his 4294967298 points.  
//! Likewise, why would you represent the smoke particle count with an `usize` when *you know* that
//! it'd be impossible for it to get past, say, 65535 ?  
//! It is my opinion, for instance, that `::std::time::Instant` should meet
//! these requirements (today, it actually doesn't because of `Redox`'s `libstd` implementation)
//! because there's no reason some platforms would be able to represent time more precisely
//! than others, not to mention the (tiny) extra memory differences - here, we'd make it consistent
//! by storing it in our side as :
//! ```
//! # fn main() {}
//! # extern crate fate;
//! # use fate::{FixedConsistentSize, FixedConsistentRepr};
//! # #[allow(dead_code)]
//! #[repr(C,packed)]
//! struct Time {
//!     secs: i64, 
//!     nanosecs: u64,
//! }
//! // Now we know we can do this :
//! unsafe impl FixedConsistentSize for Time {}
//! unsafe impl FixedConsistentRepr for Time {}
//! ```
//! TODO: derive for `FixedConsistentRepr` and `FixedConsistentSize` !
//!
//! Size consistency for a type often, but not always, implies representation consistency, which
//! is what one may actually be looking for: the `FixedConsistentRepr` trait is a stronger
//! version of `FixedConsistentSize`, which adds the requirement that the in-memory representation
//! of data is the same everywhere.  
//! For instance, a hypothetical `real32` type which may resolve to various real-number
//! implementations depending on the target, but is always 32 bits-wide, would be
//! `FixedConsistentSize` but not `FixedConsistentRepr`.
//!
//! The main use case for `FixedConsistentRepr` would be to help guaranteeing purity of bit-level
//! manipulations for some types.
//!
//! Checklist for properly implementing `FixedConsistentSize` :
//! - `enum`s and `struct`s MUST be `#[repr(C,packed)]` (`#[repr(C)]` alone is not enough because each target may have its alignment requirements);
//! - All members (if any) MUST be `FixedConsistentSize`;
//! - The type is not dynamically-sized (counter-examples include `Vec<T>`, `String`, `str`, `[T]`, `&[T]`, etc).
//! - The type is not machine-dependant (counter-examples include `#[repr(simd)]`, `isize`, `usize`, `*const T`, `*mut T`).
//!
//! All zero-sized types should implement `FixedConsistentRepr` (which implies `FixedConsistentSize`) when possible.
//!
//! # Rationales
//! - `char` is `FixedConsistentRepr` because it is always 4 bytes-wide in Rust, in case you're
//!   coming from C, C++, Java, etc.
//! - `bool` is not `FixedConsistentRepr` because its representation is unspecified, AFAIK.
//! - Tuples are not `FixedConsistentRepr` because their memory layout is unspecified, 
//!   except the empty tuple which is zero-sized. Not even (T,T,T,T) would match, because the compiler
//!   is free to remove fields if it can prove they aren't used.
//! - `f32` and `f64` are `FixedConsistentRepr` because, quoting the Rust spec, they always are 
//!   "IEEE 754-2008 `binary32` and `binary64`" formats.
//! - `Option<T>` and `Result<T,E>` *probably* are `FixedConsistentRepr` if both T and E are
//!    `FixedConsistentRepr`, but I don't know for sure.

/// A type which size is not only known at compile-time, but guaranteed to be the same
/// across operating systems, hardware, compilers and build settings.
pub unsafe trait FixedConsistentSize : Sized {}

/// A type which in-memory representation is known at compile-time and 
/// guaranteed to be the same across operating systems, hardware, compilers and build settings.
pub unsafe trait FixedConsistentRepr : FixedConsistentSize {}

macro_rules! impl_fcs_and_fcr {
    (generic for $($ty:tt)+) => {
        $(unsafe impl<T: FixedConsistentSize> FixedConsistentSize for $ty<T> {})+
        $(unsafe impl<T: FixedConsistentRepr> FixedConsistentRepr for $ty<T> {})+
    };
    (array $($i:expr)+) => {
        $(unsafe impl<T: FixedConsistentSize> FixedConsistentSize for [T; $i] {})+
        $(unsafe impl<T: FixedConsistentRepr> FixedConsistentRepr for [T; $i] {})+
    };
}
/*
macro_rules! impl_fcs {
    (basic for $($ty:ty)+) => {
        $(unsafe impl FixedConsistentSize for $ty {})+
    };
}
*/
macro_rules! impl_fcr {
    (basic for $($ty:ty)+) => {
        $(unsafe impl FixedConsistentSize for $ty {})+
        $(unsafe impl FixedConsistentRepr for $ty {})+
    };
}

use std::marker::PhantomData;
use std::num::Wrapping;

impl_fcr!(basic for () char u8 i8 u16 i16 u32 i32 u64 i64 i128 u128 f32 f64);
impl_fcs_and_fcr!(array 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32);
impl_fcs_and_fcr!(generic for PhantomData Wrapping);
