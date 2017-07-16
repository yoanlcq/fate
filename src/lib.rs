//! Fight And Travel (Game) Engine.
//!
//! [It started in C](https://github.com/yoanlcq/fate-c), it will end in Rust.

//#![deny(missing_docs)]
#![doc(test(attr(deny(warnings))))]
#![feature(test)]
#![feature(inclusive_range, inclusive_range_syntax)]
#![feature(i128_type)]
#![feature(link_llvm_intrinsics)]
//#![feature(repr_simd)]

extern crate test;
extern crate vek;

pub use vek::*;

pub mod intrin;
pub use intrin::*;
pub mod marker;
pub use marker::*;
