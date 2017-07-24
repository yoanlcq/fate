//! Generic linear algebra focused on computer graphics and intent.
//!
//! It does not try to be extensive (e.g rect primitives don't ship with a physics engine.)
//!
//! DO NOT USE (yet). This is very much a work-in progress, breaking changes happen all the time on a whim.

// TODO index vectors by range
// TODO put must_use where relevant (seems to work on functions)
// TODO tests
// TODO benchmarks
// TODO doc
// TODO provide efficient functions for AoS and SoA


#![doc(html_root_url = "https://docs.rs/vek/0.1.0")]
//#![deny(missing_docs)]
#![no_std]
#![doc(test(attr(deny(warnings))))]
#![feature(test)]
#![feature(repr_simd)]

extern crate test;

pub mod bezier;
pub use bezier::*;
pub mod clamp;
pub use clamp::*;
pub mod color_component;
pub use color_component::*;
pub mod geom;
pub use geom::*;
pub mod lerp;
pub use lerp::*;
pub mod mat;
pub use mat::*;
pub mod quat;
pub use quat::*;
pub mod tween;
pub use tween::*;
pub mod vec;
pub use vec::*;
pub mod wrap;
pub use wrap::*;
