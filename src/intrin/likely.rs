//! Branch prediction intrinsics and utilities - currently wrapping LLVM's `expect` intrinsic.

#![cfg_attr(feature = "cargo-clippy", allow(inline_always))]

extern {
    #[inline(always)]
    #[link_name = "llvm.expect.i1"]
    fn llvm_expect_i1(val: bool, expected : bool) -> bool;
    #[inline(always)]
    #[link_name = "llvm.expect.i32"]
    fn llvm_expect_i32(val: i32, expected : i32) -> i32;
    #[inline(always)]
    #[link_name = "llvm.expect.i64"]
    fn llvm_expect_i64(val: i64, expected : i64) -> i64;
}

/// Trait for types that can provide branch prediction information through LLVM's `expect` intrinsic.
pub trait Expect {
    #[inline(always)]
    /// Provides a hint that `self` is likely to equal `expected`. Always returns `self`.
    fn expect(self, expected : Self) -> Self;
}

use std::num::Wrapping;

macro_rules! impl_expect {
    (i1 for $($t:ty)+) => {
        $(
            impl Expect for $t {
                #[inline(always)]
                fn expect(self, expected : Self) -> Self {
                    unsafe { llvm_expect_i1(self as bool, expected as bool) as Self }
                }
            }
        )+
    };
    (i32 for $($t:ty)+) => {
        $(
            impl Expect for $t {
                #[inline(always)]
                fn expect(self, expected : Self) -> Self {
                    unsafe { llvm_expect_i32(self as i32, expected as i32) as Self }
                }
            }
            impl Expect for Wrapping<$t> {
                #[inline(always)]
                fn expect(self, expected : Self) -> Self {
                    unsafe { Wrapping(llvm_expect_i32(self.0 as i32, expected.0 as i32) as $t) }
                }
            }
        )+
    };
    (i64 for $($t:ty)+) => {
        $(
            impl Expect for $t {
                #[inline(always)]
                fn expect(self, expected : Self) -> Self {
                    unsafe { llvm_expect_i64(self as i64, expected as i64) as Self }
                }
            }
            impl Expect for Wrapping<$t> {
                #[inline(always)]
                fn expect(self, expected : Self) -> Self {
                    unsafe { Wrapping(llvm_expect_i64(self.0 as i64, expected.0 as i64) as $t) }
                }
            }
        )+
    };
}

impl_expect!(i1 for bool);
impl_expect!(i32 for i8 u8 i16 u16 i32 u32);
impl_expect!(i64 for i64 u64 isize usize);

/// Provides a hint that `val` is likely equal to `expected`, and returns `val`.
#[inline(always)]
pub fn expect<T: Expect>(val: T, expected: T) -> T {
    val.expect(expected)
}
/// Provides a hint that `pred` is likely to be true, and returns `pred`.
///
/// A matching intrinsic is `core::intrinsics::likely`.
#[inline(always)]
pub fn likely(pred: bool) -> bool { expect(pred, true) }
/// Provides a hint that `pred` is unlikely to be true, and returns `pred`.
///
/// A matching intrinsic is `core::intrinsics::unlikely`.
#[inline(always)]
pub fn unlikely(pred: bool) -> bool { expect(pred, false) }

/// Trait for types on which both [`likely()`](fn.likely.html) and
/// [`unlikely()`](fn.unlikely.html) can be called.
pub trait LikelyExt {
    /// Provides a hint that `self` is likely to be true, and returns `self`.
    #[inline(always)]
    fn likely(self) -> Self;
    /// Provides a hint that `self` is unlikely to be true, and returns `self`.
    #[inline(always)]
    fn unlikely(self) -> Self;
}
impl LikelyExt for bool {
    #[inline(always)]
    fn likely(self) -> Self { likely(self) }
    #[inline(always)]
    fn unlikely(self) -> Self { unlikely(self) }
}

#[cfg(test)]
mod test {
    use ::test::Bencher;
    use ::rand::random;
    use super::{likely, unlikely, expect, LikelyExt};

    #[test]
    fn likely_returns_val() {
        assert_eq!(true,  likely(true));
        assert_eq!(false, likely(false));
        assert_eq!(true,  unlikely(true));
        assert_eq!(false, unlikely(false));
        assert_eq!(false, false.unlikely());
        assert_eq!(true,  expect(true , true));
        assert_eq!(false, expect(false, false));
        assert_eq!(true,  expect(true , false));
        assert_eq!(false, expect(false, true));
        assert_eq!(42,    expect(42, 42));
        assert_eq!(42,    expect(42, 13));
        assert_eq!(42i64, expect(42i64, 42));
        assert_eq!(42i64, expect(42i64, 13));
    }

    const CNT : usize = 1<<10;
    type I = i32;
    const MAX : I = 100 as I;
    const UNDER : I = MAX - 5;

    #[bench]
    fn probably_with_likely(bench : &mut Bencher) {
        bench.iter(|| {
            let mut accum = 0 as I;
            for _ in 0..CNT {
                let val = random::<I>() % MAX;
                if likely(val < UNDER) {
                    accum += val;
                }
            }
            accum
        })
    }
    #[bench]
    fn probably_without_likely(bench : &mut Bencher) {
        bench.iter(|| {
            let mut accum = 0 as I;
            for _ in 0..CNT {
                let val = random::<I>() % MAX;
                if val < UNDER {
                    accum += val;
                }
            }
            accum
        })
    }
}
