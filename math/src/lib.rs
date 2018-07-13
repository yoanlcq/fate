extern crate vek;

pub use vek::{
    ops::*,
    vec::repr_c::*,
    vec::repr_simd::{
        Vec4 as Simd4,
        Vec8 as Simd8,
        Vec16 as Simd16,
        Vec32 as Simd32,
        Vec64 as Simd64,
    },
    mat::repr_c::column_major::*,
    mat::repr_simd::column_major::{Mat4 as SimdMat4},
    quaternion::*,
    transition::*,
    transform::*,
    bezier::*,
    geom::*,
};

