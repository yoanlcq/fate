//! The `Deterministic` and `DeterministicOrDiverges` marker traits.
//!
//! Operations on one or more [`Deterministic`](trait.Deterministic.html) data type values are
//! guaranteed to give consistent (read: the exact same) results across hardware,
//! operating systems, compilers, and build settings (in particular, optimization options).
//! The benefits are mostly cross-platform reproducibilty ("replayability") and
//! consistency of implicitly shared state over time (which is great for games).
//!
//! For instance, `std::num::Wrapping<u32>` implements [`Deterministic`](trait.Deterministic.html)
//! because `1 + 2` will always yield `3`,
//! and even `std::u32::MAX + 1` will always yield `0`.
//! Also, integer division by zero either
//! causes a panic or aborts the program, so this is treated as "consistent" behaviour for
//! the use cases [`Deterministic`](trait.Deterministic.html) was written for.
//!
//! On the other hand, `f32` and `f64` certainly do NOT implement
//! [`Deterministic`](trait.Deterministic.html), because the
//! result of, say, `cos(2.354)` depends too much on optimization options, the target hardware,
//! hardware-specific FP modes which might be set at runtime, and finally the `cos()`
//! implementation, which I wouldn't trust at all to be the same on all systems (provided that the 
//! compiler's `cos()` built-in doesn't kick in first).
//!
//! Actually, anything related to native floating-point is a natural source of inconsistency.
//! There are many articles on the subject, the following one being the most widely cited :
//! [http://gafferongames.com/networking-for-game-programmers/floating-point-determinism/](http://gafferongames.com/networking-for-game-programmers/floating-point-determinism/)
//!
//! Admittedly, `Deterministic` trait bounds are not something you would put all over your code
//! in the hope it'll make everything replayable - the purpose is merely to help making such
//! guarantees. Engine-provided time-rewinding mechanics, for instance, might want to use it.
//!
//! # Trivia
//! - Refraining from using native floating-point number types is often enough to get rid of 
//!   inconsistency, but keep in mind there are subtle ways inconsistency can creep back into your 
//!   code : for instance, some kind of concurrent update where the order in which operations are executed does matter.
//! - However, that doesn't mean that the floating-point representation itself isn't incompatible with 
//!   [`Deterministic`](trait.Deterministic.html)'s requirements. 
//!   "Soft float" types should implement `Deterministic` if they do match the mentioned consistency criteria. 
//! - Most (if not all) of the types that derive/implement automatically `std::hash::Hash` are 
//!   good candidates for [`Deterministic`](trait.Deterministic.html). NOTE: Actually, function pointers
//!   are a good counter-example. 
//! - `Deterministic` shouldn't be implemented on function/closure types because there's no 
//!   way to require that they have no side effect (and as of Rust 1.18-nightly, we can't implement traits for 
//!   `const fn`s);
//! - Operations involving both `Deterministic` and non-`Deterministic` types are non-`Deterministic`. 
//!   `Deterministic` can't prevent you from shooting yourself in the foot, but makes it a bit harder. 
//!
//! # Example (trivial) use case
//! ```rust,no_run
//! extern crate fate;
//! // Requires the latest num from the git repo as of today (20th april 2017)
//! extern crate num;
//!
//! use fate::{Deterministic, Xy};
//! use num::Num;
//! use std::num::Wrapping as Wp;
//!
//! // Work in our own deterministic space. Maybe this is a networked game
//! // in which only player input is shared between peers, from which each peer updates their
//! // their local copy of the game's state (some kind of Peer-to-Peer deterministic lockstep mechanic).
//! fn update_logic<T>(pos: &mut Xy<T>) where T: Copy + Num + Deterministic {
//!     pos.x = pos.x + pos.y;
//! }
//! // On the other hand, _rendering_ has no effect on the game's state, so we don't care if it's
//! // inconsistent across peers. It's all the better anyway because GPUs love 32-bit
//! // floating point types, and so do 3D APIs.
//! fn update_rendering(_: Xy<f32>) {
//!     unimplemented!()
//! }
//!
//! fn main() {
//!     let mut player = Xy::new(Wp(42_i32), Wp(42_i32));
//!     loop {
//!         update_logic(&mut player);
//!         update_rendering(Xy::new(player.x.0 as f32 / 100f32, 0_f32)); // Whatever
//!     }
//! }
//! ```

/// Operations on one or more `Deterministic` data type values are guaranteed to give
/// consistent (read: the exact same) results across hardware,
/// operating systems, compilers, and build settings (in particular, optimization options).
///
/// This trait is `unsafe` because it asserts properties that can't be proved to
/// hold true, except by careful programmers.
pub unsafe trait Deterministic {}

/// A type that's close to meeting [`Deterministic`](trait.Deterministic.html)'s requirements,
/// because its only "inconsistent" behaviours (if any) are to diverge (e.g panic).
/// Please don't confuse this with "consistent" divergence (e.g integer divide by zero).
///
/// This was intended for primitive integer types because basic operations may panic if they
/// overflow in debug builds.
///
/// As per the description above :
/// - `DeterministicOrDiverges` does NOT imply `Deterministic`;
/// - `Deterministic` does NOT imply `DeterministicOrDiverges` either.
pub unsafe trait DeterministicOrDiverges {}

macro_rules! impl_deterministic {
    (basic_or_diverges for $($ty:ty)+) => {
        $(unsafe impl DeterministicOrDiverges for $ty {})+
    };
    (basic for $($ty:ty)+) => {
        $(unsafe impl DeterministicOrDiverges for $ty {})+
        $(unsafe impl Deterministic for $ty {})+
    };
    (wrapping for $($ty:ty)+) => {
        $(unsafe impl DeterministicOrDiverges for Wrapping<$ty> {})+
        $(unsafe impl Deterministic for Wrapping<$ty> {})+
    };
    (generic for $($tt:tt)+) => {
        $(unsafe impl<T: DeterministicOrDiverges> DeterministicOrDiverges for $tt<T> {})+
        $(unsafe impl<T: Deterministic> Deterministic for $tt<T> {})+
    };
    (array $($i:expr)+) => {
        $(unsafe impl<T: DeterministicOrDiverges> DeterministicOrDiverges for [T; $i] {})+
        $(unsafe impl<T: Deterministic> Deterministic for [T; $i] {})+
    };
    (tuple $($T:ident)+) => {
        unsafe impl<$($T: DeterministicOrDiverges),*> DeterministicOrDiverges for ($($T,)+) {}
        unsafe impl<$($T: Deterministic),*> Deterministic for ($($T,)+) {}
    };
}

use ::std::num::Wrapping;
use ::std::ops::*;
use ::std::cmp::Ordering;
use ::std::any::TypeId;
use ::std::marker::PhantomData;
use ::std::sync::Arc;
use ::std::rc::Rc;

unsafe impl<T: Deterministic> Deterministic for [T] {}
unsafe impl<T: DeterministicOrDiverges> DeterministicOrDiverges for [T] {}
impl_deterministic!(array 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32);
impl_deterministic!(tuple A);
impl_deterministic!(tuple A B);
impl_deterministic!(tuple A B C);
impl_deterministic!(tuple A B C D);
impl_deterministic!(tuple A B C D E);
impl_deterministic!(tuple A B C D E F);
impl_deterministic!(tuple A B C D E F G);
impl_deterministic!(tuple A B C D E F G H);
impl_deterministic!(tuple A B C D E F G H I);
impl_deterministic!(tuple A B C D E F G H I J);

impl_deterministic!(basic for () bool char str String);
impl_deterministic!(basic for RangeFull TypeId Ordering);

impl_deterministic!(wrapping for u8 i8 u16 i16 u32 i32 u64 i64 usize isize);
#[feature(i128_type)]
impl_deterministic!(wrapping for i128 u128);

impl_deterministic!(basic_or_diverges for u8 i8 u16 i16 u32 i32 u64 i64 usize isize);
#[feature(i128_type)]
impl_deterministic!(basic_or_diverges for i128 u128);

unsafe impl<T> Deterministic for *const T {}
unsafe impl<T> Deterministic for *mut T {}

impl_deterministic!(generic for Box Rc Arc Vec);
impl_deterministic!(generic for Range RangeFrom RangeTo);
#[feature(inclusive_range)]
impl_deterministic!(generic for RangeInclusive RangeToInclusive);

unsafe impl<T: ?Sized + Deterministic> Deterministic for PhantomData<T> {}
unsafe impl<T: Deterministic> Deterministic for Option<T> {}
unsafe impl<T: DeterministicOrDiverges> DeterministicOrDiverges for Option<T> {}
unsafe impl<T,E> Deterministic for Result<T,E> where T: Deterministic, E: Deterministic {}
unsafe impl<T,E> DeterministicOrDiverges for Result<T,E> where T: DeterministicOrDiverges, E: DeterministicOrDiverges {}

#[cfg(test)]
mod tests {
    use super::Deterministic;
    use ::std::num::Wrapping as Wp;

    // Marking a type as deterministic, the regular way.
    struct Well<T>(T);
    unsafe impl<T: Deterministic> Deterministic for Well<T> {}

    // Coming soon!
    // A type can derive Deterministic if and only if all of its members
    // are Deterministic.
    // TODO
    // #[derive(Deterministic)]
    // struct Nice<T>(T);

    fn require_deterministic<T>(_: &T) where T: Deterministic {}

    #[test]
    fn should_not_accept_deterministic() {
        // The day this compiles, we're in trouble.
        // require_deterministic(&42_f32);
        // require_deterministic(&vec![42_f32, 13_f32]);
    }
    #[test]
    fn should_accept_deterministic() {
        require_deterministic(&Wp(42));
        require_deterministic(&vec![Wp(42), Wp(13)]);
        require_deterministic(&Well(Wp(42)));
        // require_deterministic(&Nice(42));
    }
}
