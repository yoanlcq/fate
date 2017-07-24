
// NOTE: Most of the code here assumes that we only implement square matrices.
// Here's a rationale:
// - There's not much you can do with non-square matrices;
//   All transforms I know of only exist for square matrices.
// - You can emulate them using a higher-dimensioned matrix with
//   appropriate elements set to zero or one.
// - Who ever uses them actually ?
//
// A nice consequence is that it slightly reduces implementation complexity.

extern crate num_traits;

use self::num_traits::{Zero, One, Float, NumCast};
use core::slice;
use core::iter::FromIterator;
use core::ops::*;
use core::mem;
use core::ptr;
use vec::*;
use geom;

// TODO: move FrustumPlanes to geom module ??
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct FrustumPlanes<T> {
    pub left: T,
    pub right: T,
    pub bottom: T,
    pub top: T,
    pub near: T,
    pub far: T,
}
pub type Ortho<T> = FrustumPlanes<T>;
pub type Viewport = geom::Rect<u32, u32>;

macro_rules! mat_impl {
    ($Mat:ident, $mat:ident,
     $Row:ident, $ExactlyRow:ident, $rows:expr, 
     $Col:ident, $ExactlyCol:ident, $cols:expr,
     ($($get_row:tt)+)
     $(($MulRow:ident $MulCol:ident))+
    ) => {
        pub mod $mat {
            use super::*;

			// TODO: repr(align(16)) is good on x86. Investigate for other platforms.
            #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
			#[repr(align(16))]
            pub struct $Mat<T> { pub rows: $Row<$Col<T>> }

            impl<T: Zero + One> Default for $Mat<T> {
                fn default() -> Self {
                    Self::identity()
                }
            }

            pub type RowIter<'a, T> = slice::Iter<'a, $Row<T>>;
            pub type RowIterMut<'a, T> = slice::IterMut<'a, $Row<T>>;

            impl<T> $Mat<T> {
                pub fn zero() -> Self where T: Zero {
                    let mut out: Self = unsafe { mem::uninitialized() };
                    $(out.rows.$get_row = $Row::zero();)+
                    out
                }
                pub fn identity() -> Self where T: Zero + One {
                    let mut out = Self::zero();
                    $((out.rows.$get_row).$get_row = T::one();)+
                    out
                }
                /*
                pub fn broadcast_major_diagonal(val: T) { // TODO
                    
                }
                pub fn pow(self, n: i32) {}
                pub fn determinant(self) -> T {}
                pub fn invert_2x2() {}
                pub fn invert_3x3() {}
                pub fn rotation_from_two_vectors() {
                    axis = vs * vf;
                    angle = arccos(vs.dot(vf));
                }
                pub fn shearing_x|y|z
                pub fn from_quaternion() { /* Q54*/ }
                */
                pub fn row_count(&self) -> usize {
                    $rows
                }
                pub fn col_count(&self) -> usize {
                    $cols
                }
                pub fn as_slice(&self) -> &[$Row<T>] {
                    self.rows.as_slice()
                }
                pub fn as_mut_slice(&mut self) -> &mut [$Row<T>] {
                    self.rows.as_mut_slice()
                }
                pub fn iter(&self) -> RowIter<T> {
                    self.into_iter()
                }
                pub fn iter_mut(&mut self) -> RowIterMut<T> {
                    self.into_iter()
                }
                pub fn into_col_iter(self) -> IntoColIter<T> {
                    IntoColIter::new(self)
                }
                // WISH: _unchecked() variants.
                pub fn row<V>(self, i: usize) -> V where T: Clone, V: $ExactlyRow<T> {
                    self.rows[i].clone().into()
                }
                // NOTE: relying on this being a square matrix
                pub fn col<V>(self, i: usize) -> V where T: Clone, V: $ExactlyCol<T> {
                    let m = self.rows;
                    let mut out: $Col<T> = unsafe { mem::uninitialized() };
                    $(out.$get_row = m.$get_row[i].clone();)+
                    out.into()
                }
                pub fn mul_memberwise(self, m: Self) -> Self where T: Mul<Output=T> {
                    Self {
                        rows: $Row::new(
                            $(self.rows.$get_row * m.rows.$get_row),+
                        )
                    }
                }
                // NOTE: Ugly, we'd have to diretcly access elements instead of using
                // get_unchecked*() everytime
                pub fn transposed(self) -> Self {
                    let mut out: Self = unsafe { mem::uninitialized() };
                    for i in 0..$rows {
                        for j in 0..$cols {
                            // NOTE: bitwise copy is safe since we move all elements anyway
                            unsafe {
                                *out.rows.get_unchecked_mut(i).get_unchecked_mut(j) = 
                                    ptr::read(self.rows.get_unchecked(j).get_unchecked(i));
                            }
                        }
                    }
                    out
                }

                /*
                 * NOTE: Use this prototype instead for non-square matrix types
                pub fn mul_vec<I,O>(self, rhs: I) -> O
                    where T: Clone + Zero + Add<Output=T> + Mul<Output=T>,
                          I: $ExactlyRow<T>,
                          O: $ExactlyCol<T>
                */
                // NOTE: private function, only to (hopefully??) help the compiler reduce code size.
                // Also it's just convenient.
                fn mul_vec<V>(self, rhs: V) -> V
                    where T: Clone + Zero + Add<Output=T> + Mul<Output=T>,
                          V: $ExactlyRow<T>,
                {
                    let mut out: $Col<T> = unsafe { mem::uninitialized() };
                    let rhs = rhs.into();
                    for (out_elem, row) in out.iter_mut().zip(self.rows.into_iter()) {
                        *out_elem = row.into_iter()
                            .zip(rhs.iter().cloned())
                            .fold(T::zero(), |acc, (a,b)| acc + a*b);
                    }
                    out.into()
                }
            }

            impl<T> Mul for $Mat<T> 
                where T: Clone + Zero + Add<Output=T> + Mul<Output=T>
            {
                type Output = Self;
                fn mul(self, rhs: Self) -> Self::Output {
                    let mut out: Self = unsafe { mem::uninitialized() };
                    let rhs_transposed = rhs.transposed();
                    for (out_row, row) in out.rows.iter_mut().zip(self.rows.iter()) {
                        for (out_elem, col) in out_row.iter_mut().zip(rhs_transposed.rows.iter()) {
                            *out_elem = row.iter().cloned()
                                .zip(col.iter().cloned())
                                .fold(T::zero(), |acc, (a,b)| acc + a*b);
                        }
                    }
                    out
                }
            }
            $(
                impl<T> Mul<$MulRow<T>> for $Mat<T> 
                    where T: Clone + Zero + Add<Output=T> + Mul<Output=T>
                {
                    type Output = $MulCol<T>;
                    fn mul(self, rhs: $MulRow<T>) -> Self::Output {
                        self.mul_vec(rhs)
                    }
                }
            )+
            impl<T> Mul<T> for $Mat<T> 
                where T: Clone + Zero + Add<Output=T> + Mul<Output=T>
            {
                type Output = Self;
                fn mul(self, rhs: T) -> Self::Output {
                    Self {
                        rows: $Row::new(
                            $(self.rows.$get_row * rhs.clone()),+
                        )
                    }
                }
            }
            // TODO implement vec*mat. Not used often, but does exist.

            impl<T> MulAssign for $Mat<T>
                where T: Clone + Zero + Add<Output=T> + Mul<Output=T>
            { 
                fn mul_assign(&mut self, rhs: Self) { *self = self.clone() * rhs; }
            }

            impl<T> MulAssign<T> for $Mat<T>
                where T: Clone + Zero + Add<Output=T> + Mul<Output=T>
            { 
                fn mul_assign(&mut self, rhs: T) { *self = self.clone() * rhs; }
            }


            impl<T> Add for $Mat<T> where T: Add<Output=T> {
                type Output = Self;
                fn add(self, rhs: Self) -> Self::Output {
                    Self {
                        rows: $Row::new(
                            $(self.rows.$get_row + rhs.rows.$get_row),+
                        )
                    }
                }
            }
            impl<T> Sub for $Mat<T> where T: Sub<Output=T> {
                type Output = Self;
                fn sub(self, rhs: Self) -> Self::Output {
                    Self {
                        rows: $Row::new(
                            $(self.rows.$get_row - rhs.rows.$get_row),+
                        )
                    }
                }
            }
            impl<T> Div for $Mat<T> where T: Div<Output=T> {
                type Output = Self;
                fn div(self, rhs: Self) -> Self::Output {
                    Self {
                        rows: $Row::new(
                            $(self.rows.$get_row / rhs.rows.$get_row),+
                        )
                    }
                }
            }
            impl<T> Rem for $Mat<T> where T: Rem<Output=T> {
                type Output = Self;
                fn rem(self, rhs: Self) -> Self::Output {
                    Self {
                        rows: $Row::new(
                            $(self.rows.$get_row % rhs.rows.$get_row),+
                        )
                    }
                }
            }
            impl<T> Neg for $Mat<T> where T: Neg<Output=T> {
                type Output = Self;
                fn neg(self) -> Self::Output {
                    Self {
                        rows: $Row::new(
                            $(-self.rows.$get_row),+
                        )
                    }
                }
            }

            impl<T> Add<T> for $Mat<T> where T: Clone + Add<Output=T> {
                type Output = Self;
                fn add(self, rhs: T) -> Self::Output {
                    Self {
                        rows: $Row::new(
                            $(self.rows.$get_row + rhs.clone()),+
                        )
                    }
                }
            }
            impl<T> Sub<T> for $Mat<T> where T: Clone + Sub<Output=T> {
                type Output = Self;
                fn sub(self, rhs: T) -> Self::Output {
                    Self {
                        rows: $Row::new(
                            $(self.rows.$get_row - rhs.clone()),+
                        )
                    }
                }
            }
            impl<T> Div<T> for $Mat<T> where T: Clone + Div<Output=T> {
                type Output = Self;
                fn div(self, rhs: T) -> Self::Output {
                    Self {
                        rows: $Row::new(
                            $(self.rows.$get_row / rhs.clone()),+
                        )
                    }
                }
            }
            impl<T> Rem<T> for $Mat<T> where T: Clone + Rem<Output=T> {
                type Output = Self;
                fn rem(self, rhs: T) -> Self::Output {
                    Self {
                        rows: $Row::new(
                            $(self.rows.$get_row % rhs.clone()),+
                        )
                    }
                }
            }

            impl<T: Add<Output=T> + Clone> AddAssign    for $Mat<T> { fn add_assign(&mut self, rhs: Self) { *self = self.clone() + rhs; } }
            impl<T: Add<Output=T> + Clone> AddAssign<T> for $Mat<T> { fn add_assign(&mut self, rhs: T   ) { *self = self.clone() + rhs; } }
            impl<T: Sub<Output=T> + Clone> SubAssign    for $Mat<T> { fn sub_assign(&mut self, rhs: Self) { *self = self.clone() - rhs; } }
            impl<T: Sub<Output=T> + Clone> SubAssign<T> for $Mat<T> { fn sub_assign(&mut self, rhs: T   ) { *self = self.clone() - rhs; } }
            impl<T: Div<Output=T> + Clone> DivAssign    for $Mat<T> { fn div_assign(&mut self, rhs: Self) { *self = self.clone() / rhs; } }
            impl<T: Div<Output=T> + Clone> DivAssign<T> for $Mat<T> { fn div_assign(&mut self, rhs: T   ) { *self = self.clone() / rhs; } }
            impl<T: Rem<Output=T> + Clone> RemAssign    for $Mat<T> { fn rem_assign(&mut self, rhs: Self) { *self = self.clone() % rhs; } }
            impl<T: Rem<Output=T> + Clone> RemAssign<T> for $Mat<T> { fn rem_assign(&mut self, rhs: T   ) { *self = self.clone() % rhs; } }


            impl<T> IndexMut<usize> for $Mat<T> {
                fn index_mut(&mut self, i: usize) -> &mut Self::Output {
                    &mut self.rows[i]
                }
            }
            impl<T> Index<usize> for $Mat<T> {
                type Output = $Row<T>;
                fn index(&self, i: usize) -> &Self::Output {
                    &self.rows[i]
                }
            }

            #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
            pub struct IntoRowIter<T> { m: $Mat<T>, i: usize }
            #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
            pub struct IntoColIter<T> { m: $Mat<T>, j: usize }
            impl<T> IntoRowIter<T> { fn new(m: $Mat<T>) -> Self { Self { m, i: 0 } } }
            impl<T> IntoColIter<T> { fn new(m: $Mat<T>) -> Self { Self { m, j: 0 } } }

            impl<T> Iterator for IntoRowIter<T> {
                type Item = $Row<T>;
                fn next(&mut self) -> Option<Self::Item> {
                    let out = self.m.rows.as_slice().get(self.i);
                    self.i += 1;
                    out.map(|x| unsafe { ptr::read(x) }) // XXX might want to use read_unaligned() ?
                }
                fn size_hint(&self) -> (usize, Option<usize>) {
                    let rem = $rows - self.i;
                    (rem, Some(rem))
                }
            }

            impl<T> Iterator for IntoColIter<T> {
                type Item = $Col<T>;
                fn next(&mut self) -> Option<Self::Item> {
                    if self.j >= $cols {
                        return None;
                    }
                    let mut out: $Col<T> = unsafe { mem::uninitialized() };
                    // NOTE: assuming a square matrix again
                    $(
                        out.$get_row = unsafe { 
                            ptr::read(
                                self.m.rows.$get_row.get_unchecked(self.j)
                            )
                        };
                    )+ 
                    self.j += 1;
                    Some(out)
                }
                fn size_hint(&self) -> (usize, Option<usize>) {
                    let rem = $cols - self.j;
                    (rem, Some(rem))
                }
            }
            impl<T> ExactSizeIterator for IntoRowIter<T> { fn len(&self) -> usize { $rows - self.i } }
            impl<T> ExactSizeIterator for IntoColIter<T> { fn len(&self) -> usize { $cols - self.j } }

            impl<T> IntoIterator for $Mat<T> {
                type Item = $Row<T>;
                type IntoIter = IntoRowIter<T>;
                fn into_iter(self) -> Self::IntoIter {
                    Self::IntoIter::new(self)
                }
            }
            impl<'a, T> IntoIterator for &'a $Mat<T> {
                type Item = &'a $Row<T>;
                type IntoIter = RowIter<'a, T>;
                fn into_iter(self) -> Self::IntoIter {
                    self.as_slice().into_iter()
                }
            }
            impl<'a, T> IntoIterator for &'a mut $Mat<T> {
                type Item = &'a mut $Row<T>;
                type IntoIter = RowIterMut<'a, T>;
                fn into_iter(self) -> Self::IntoIter {
                    self.as_mut_slice().into_iter()
                }
            }

            impl<T: Zero + One> FromIterator<$Row<T>> for $Mat<T> {
                fn from_iter<I>(iter: I) -> Self where I: IntoIterator<Item=$Row<T>> {
                    let mut out = Self::default();
                    let mut intoiter = iter.into_iter();
                    for out_row in &mut out {
                        if let Some(in_row) = intoiter.next() {
                            *out_row = in_row
                        } else {
                            break;
                        }
                    }
                    out
                }
            }
        }
        pub use $mat::$Mat;
    }
}

mat_impl!(Mat2, mat2, Vec2, Exactly2, 2, Vec2, Exactly2, 2, (0 1    ) (Vec2 Vec2) (Xy Xy) (Uv Uv) (Extent2 Extent2));
mat_impl!(Mat3, mat3, Vec3, Exactly3, 3, Vec3, Exactly3, 3, (0 1 2  ) (Vec3 Vec3) (Xyz Xyz) (Rgb Rgb) (Uvw Uvw) (Extent3 Extent3));
mat_impl!(Mat4, mat4, Vec4, Exactly4, 4, Vec4, Exactly4, 4, (0 1 2 3) (Vec4 Vec4) (Xyzw Xyzw) (Rgba Rgba));


impl<T> Mat2<T> {
    pub fn rotation_z(angle_radians: T) -> Self where T: Float {
        let c = angle_radians.cos();
        let s = angle_radians.sin();
        Self {
            rows: Vec2(
                Vec2(c, -s),
                Vec2(s,  c)
            )
        }
    }
    pub fn scaling_2d<V: Into<Xy<T>>>(v: V) -> Self where T: Zero {
        let Xy { x, y } = v.into();
        Self {
            rows: Vec2(
                Vec2(x, T::zero()),
                Vec2(T::zero(), y)
            )
        }
    }
    pub fn shear_x(k: T) -> Self where T: Zero + One {
        Self {
            rows: Vec2(
                Vec2(T::one(), k),
                Vec2(T::zero(), T::one())
            )
        }
    }
    pub fn shear_y(k: T) -> Self where T: Zero + One {
        Self {
            rows: Vec2(
                Vec2(T::one(), T::zero()),
                Vec2(k, T::one())
            )
        }
    }
}

impl<T> Mat3<T> {
    pub fn translation_2d<V: Into<Xy<T>>>(v: V) -> Self where T: Zero + One {
        let v = v.into();
        let mut out = Self::identity();
        (out.rows.0).2 = v.x;
        (out.rows.1).2 = v.y;
        out
    }
    pub fn scaling_3d<V: Into<Xyz<T>>>(v: V) -> Self where T: Zero {
        let Xyz { x, y, z } = v.into();
        Self {
            rows: Vec3(
                Vec3(x, T::zero(), T::zero()),
                Vec3(T::zero(), y, T::zero()),
                Vec3(T::zero(), T::zero(), z)
            )
        }
    }
    pub fn rotation_x(angle_radians: T) -> Self where T: Float {
        let c = angle_radians.cos();
        let s = angle_radians.sin();
        Self {
            rows: Vec3(
                Vec3(T::one(), T::zero(), T::zero()),
                Vec3(T::zero(), c, -s),
                Vec3(T::zero(), s, c)
            )
        }
    }
    pub fn rotation_y(angle_radians: T) -> Self where T: Float {
        let c = angle_radians.cos();
        let s = angle_radians.sin();
        Self {
            rows: Vec3(
                Vec3(c, T::zero(), s),
                Vec3(T::zero(), T::one(), T::zero()),
                Vec3(-s, T::zero(), c)
            )
        }
    }
    pub fn rotation_z(angle_radians: T) -> Self where T: Float {
        Mat2::rotation_z(angle_radians).into()
    }

    pub fn rotation_3d<V: Into<Xyz<T>>>(angle_radians: T, axis: V) -> Self where T: Float {
        let Xyz { x, y, z } = axis.into().normalized();
        let s = angle_radians.sin();
        let c = angle_radians.cos();
        let oc = T::one() - c;
        Self {
            rows: Vec3(
                Vec3(oc*x*x + c  , oc*x*y - z*s, oc*z*x + y*s),
                Vec3(oc*x*y + z*s, oc*y*y + c  , oc*y*z - x*s),
                Vec3(oc*z*x - y*s, oc*y*z + x*s, oc*z*z + c  )
            )
        }
    }
}

impl<T> Mat4<T> {

    //
    // BASIC
    //

    // Taken verbatim from datenwolf's linmath.h
    // As mentioned in the original, it assumes that the matrix is invertible.
    pub fn inverted(self) -> Self where T: Float
    {
        let mut m = self.rows;
        let s = (
            (m.0).0*(m.1).1 - (m.1).0*(m.0).1,
            (m.0).0*(m.1).2 - (m.1).0*(m.0).2,
            (m.0).0*(m.1).3 - (m.1).0*(m.0).3,
            (m.0).1*(m.1).2 - (m.1).1*(m.0).2,
            (m.0).1*(m.1).3 - (m.1).1*(m.0).3,
            (m.0).2*(m.1).3 - (m.1).2*(m.0).3,
        );
        let c = (
            (m.2).0*(m.3).1 - (m.3).0*(m.2).1,
            (m.2).0*(m.3).2 - (m.3).0*(m.2).2,
            (m.2).0*(m.3).3 - (m.3).0*(m.2).3,
            (m.2).1*(m.3).2 - (m.3).1*(m.2).2,
            (m.2).1*(m.3).3 - (m.3).1*(m.2).3,
            (m.2).2*(m.3).3 - (m.3).2*(m.2).3,
        );
        
        let idet = T::one() / ( s.0*c.5-s.1*c.4+s.2*c.3+s.3*c.2-s.4*c.1+s.5*c.0 );
        
        (m.0).0 = ( (m.1).1 * c.5 - (m.1).2 * c.4 + (m.1).3 * c.3) * idet;
        (m.0).1 = (-(m.0).1 * c.5 + (m.0).2 * c.4 - (m.0).3 * c.3) * idet;
        (m.0).2 = ( (m.3).1 * s.5 - (m.3).2 * s.4 + (m.3).3 * s.3) * idet;
        (m.0).3 = (-(m.2).1 * s.5 + (m.2).2 * s.4 - (m.2).3 * s.3) * idet;
        (m.1).0 = (-(m.1).0 * c.5 + (m.1).2 * c.2 - (m.1).3 * c.1) * idet;
        (m.1).1 = ( (m.0).0 * c.5 - (m.0).2 * c.2 + (m.0).3 * c.1) * idet;
        (m.1).2 = (-(m.3).0 * s.5 + (m.3).2 * s.2 - (m.3).3 * s.1) * idet;
        (m.1).3 = ( (m.2).0 * s.5 - (m.2).2 * s.2 + (m.2).3 * s.1) * idet;
        (m.2).0 = ( (m.1).0 * c.4 - (m.1).1 * c.2 + (m.1).3 * c.0) * idet;
        (m.2).1 = (-(m.0).0 * c.4 + (m.0).1 * c.2 - (m.0).3 * c.0) * idet;
        (m.2).2 = ( (m.3).0 * s.4 - (m.3).1 * s.2 + (m.3).3 * s.0) * idet;
        (m.2).3 = (-(m.2).0 * s.4 + (m.2).1 * s.2 - (m.2).3 * s.0) * idet;
        (m.3).0 = (-(m.1).0 * c.3 + (m.1).1 * c.1 - (m.1).2 * c.0) * idet;
        (m.3).1 = ( (m.0).0 * c.3 - (m.0).1 * c.1 + (m.0).2 * c.0) * idet;
        (m.3).2 = (-(m.3).0 * s.3 + (m.3).1 * s.1 - (m.3).2 * s.0) * idet;
        (m.3).3 = ( (m.2).0 * s.3 - (m.2).1 * s.1 + (m.2).2 * s.0) * idet;

        Self { rows: m }
    }

    pub fn orthonormalized(self) -> Self where T: Float {
        let mut r = self.rows;

        r.2 = r.2.normalized();

        let s = r.1.dot(r.2);
        let h = r.2 * s;
        r.1 -= h;
        // r.2 = r.2.normalized();

        // let s = r.1.dot(r.2);
        // let h = r.2 * s;
        r.1 -= h;
        r.1 = r.1.normalized();

        let s = r.0.dot(r.1);
        let h = r.1 * s;
        r.0 -= h;
        r.0 = r.0.normalized();

        Self { rows: r }
    }

    //
    // TRANSFORMS
    //

    pub fn translation_2d<V: Into<Xy<T>>>(v: V) -> Self where T: Zero + One {
        Mat3::translation_2d(v).into()
    }
    pub fn translation_3d<V: Into<Xyz<T>>>(v: V) -> Self where T: Zero + One {
        let v = v.into();
        let mut out = Self::identity();
        (out.rows.0).3 = v.x;
        (out.rows.1).3 = v.y;
        (out.rows.2).3 = v.z;
        out
    }
    pub fn translate_in_place_3d<V: Into<Xyz<T>>>(&mut self, v: V) where T: Clone + Zero + One + AddAssign {
        let Xyz { x, y, z } = v.into();
        let t = Xyzw { x, y, z, w: T::zero() };
        (self.rows.3).0 += self.rows.0.clone().dot(t.clone());
        (self.rows.3).1 += self.rows.1.clone().dot(t.clone());
        (self.rows.3).2 += self.rows.2.clone().dot(t.clone());
        (self.rows.3).3 += self.rows.3.clone().dot(t.clone());
    }

    pub fn scaling_3d<V: Into<Xyz<T>>>(v: V) -> Self where T: Zero + One {
        Mat3::scaling_3d(v).into()
    }
    pub fn rotation_x(angle_radians: T) -> Self where T: Float {
        Mat3::rotation_x(angle_radians).into()
    }
    pub fn rotation_y(angle_radians: T) -> Self where T: Float {
        Mat3::rotation_y(angle_radians).into()
    }
    pub fn rotation_z(angle_radians: T) -> Self where T: Float {
        Mat3::rotation_z(angle_radians).into()
    }
    pub fn rotation_3d<V: Into<Xyz<T>>>(angle_radians: T, axis: V) -> Self where T: Float {
        Mat3::rotation_3d(angle_radians, axis).into()
    }

    //
    // VIEW
    //

    pub fn look_at<V: Into<Xyz<T>>>(eye: V, center: V, up: V) -> Self
        where T: Float + AddAssign
    {
        let (eye, center, up) = (eye.into(), center.into(), up.into());
        let f = (center - eye).normalized();
        let s = f.cross(up).normalized();
        let t = s.cross(f);
        let mut out = Self::identity();
        (out.rows.0).0 =  s.x;
        (out.rows.0).1 =  t.x;
        (out.rows.0).2 = -f.x;
        (out.rows.1).0 =  s.y;
        (out.rows.1).1 =  t.y;
        (out.rows.1).2 = -f.y;
        (out.rows.2).0 =  s.z;
        (out.rows.2).1 =  t.z;
        (out.rows.2).2 = -f.z;
        out.translate_in_place_3d(-eye);
        out
    }


    //
    // PROJECTIONS
    //

    pub fn orthographic (o: Ortho<T>) -> Self
        where T: Copy + Zero + One 
               + Add<Output=T> + Sub<Output=T> + Neg<Output=T>
               + Mul<Output=T> + Div<Output=T>
    {
        let Ortho { left: l, right: r, bottom: b, top: t, near: n, far: f } = o;

        let two = T::one() + T::one();
        let mut out = Self::zero();
        (out.rows.0).0 = two/(r-l);
        (out.rows.1).1 = two/(t-b);
        (out.rows.2).2 = -two/(f-n);
        (out.rows.3).0 = -(r+l)/(r-l);
        (out.rows.3).1 = -(t+b)/(t-b);
        (out.rows.3).2 = -(f+n)/(f-n);
        (out.rows.3).3 = T::one();
        out
    }
    pub fn perspective (fov_y_radians: T, aspect_ratio: T, near: T, far: T) -> Self 
        where T: Float
    {
        assert!(fov_y_radians > T::zero());
        let two = T::one() + T::one();
        let a = T::one() / ((fov_y_radians / two).tan());
        let mut out = Self::zero();
        (out.rows.0).0 = a / aspect_ratio;
        (out.rows.1).1 = a;
        (out.rows.2).2 = -((far + near) / (far - near));
        (out.rows.2).3 = -T::one();
        (out.rows.3).2 = -((two * far * near) / (far - near));
        out
    }
    // NOTE: Right-handed
    pub fn perspective_fov (fov_y_radians: T, width: T, height: T, near: T, far: T) -> Self 
        where T: Float
    {
        assert!(width > T::zero());
        assert!(height > T::zero());
        assert!(fov_y_radians > T::zero());

        let two = T::one() + T::one();
        let rad = fov_y_radians;
        let h = (rad/two).cos() / (rad/two).sin();
        let w = h * height / width;

        let mut out = Self::zero();
        (out.rows.0).0 = w;
        (out.rows.1).1 = h;
        (out.rows.2).2 = -(far + near) / (far - near);
        (out.rows.2).3 = -T::one();
        (out.rows.3).2 = -(two * far * near) / (far - near);
        out
    }
    pub fn infinite_perspective (fov_y_radians: T, aspect_ratio: T, near: T) -> Self 
        where T: Float
    {
        Self::tweaked_infinite_perspective(fov_y_radians, aspect_ratio, near, T::zero())
    }

    pub fn tweaked_infinite_perspective (fov_y_radians: T, aspect_ratio: T, near: T, epsilon: T) -> Self 
        where T: Float
    {
        let two = T::one() + T::one();
        let range = (fov_y_radians / two).tan() * near;
        let left = -range * aspect_ratio;
        let right = range * aspect_ratio;
        let bottom = -range;
        let top = range;

        let mut out = Self::zero();
        (out.rows.0).0 = (two * near) / (right - left);
        (out.rows.1).1 = (two * near) / (top - bottom);
        (out.rows.2).2 = epsilon - T::one();
        (out.rows.2).3 = -T::one();
        (out.rows.3).2 = (epsilon - two) * near;
        out
    }
    pub fn frustum (o: FrustumPlanes<T>) -> Self 
        where T: Copy + Zero + One 
               + Add<Output=T> + Sub<Output=T> + Neg<Output=T>
               + Mul<Output=T> + Div<Output=T>
    {
        let FrustumPlanes { left: l, right: r, bottom: b, top: t, near: n, far: f } = o;

        let two = T::one() + T::one();
        let mut out = Self::zero();
        (out.rows.0).0 = two*n/(r-l);
        (out.rows.1).1 = two*n/(t-b);
        (out.rows.2).0 = (r+l)/(r-l);
        (out.rows.2).1 = (t+b)/(t-b);
        (out.rows.2).2 = -(f+n)/(f-n);
        (out.rows.2).3 = -T::one();
        (out.rows.3).2 = -two*(f*n)/(f-n);
        out
    }

    //
    // PICKING
    //

    // GLM's pickMatrix
    pub fn picking_region<V2: Into<Xy<T>>>(center: V2, delta: V2, viewport: Viewport) -> Self
        where T: Zero + One + Copy + NumCast + PartialOrd + Sub<Output=T> + Div<Output=T>
    {
        let (center, delta, viewport) = (center.into(), delta.into(), viewport.cast(|p| T::from(p).unwrap(), |e| T::from(e).unwrap()));
        assert!(delta.x > T::zero());
        assert!(delta.y > T::zero());
        let two = T::one() + T::one();

        let tr = Vec3(
            (viewport.w() - two * (center.x - viewport.x())) / delta.x,
            (viewport.h() - two * (center.y - viewport.y())) / delta.y,
            T::zero()
        );
        let sc = Vec3(
            viewport.w() / delta.x,
            viewport.h() / delta.y,
            T::one()
        );
        Self::scaling_3d(sc) * (Self::translation_3d(tr))
    }
    pub fn project<V3>(obj: V3, modelview: Self, proj: Self, viewport: Viewport) -> V3
        where T: Zero + One + Copy + NumCast + Add<Output=T> + Mul<Output=T> + Div<Output=T>,
              V3: Into<Xyz<T>> + From<Xyz<T>>
    {
        let viewport = viewport.cast(|p| T::from(p).unwrap(), |e| T::from(e).unwrap());
        let mut tmp = Xyzw::point(obj.into());
        tmp = modelview * tmp;
        tmp = proj * tmp;

        let two = T::one() + T::one();
        let half_one = T::one() / two;

        tmp /= tmp.w;
        tmp = tmp / two + half_one;
        tmp.x = tmp.x * viewport.w() + viewport.x();
        tmp.y = tmp.y * viewport.h() + viewport.y();

        tmp.into_xyz().into()
    }
    pub fn unproject<V3>(ray: V3, modelview: Self, proj: Self, viewport: Viewport) -> V3
        where T: Float,
              V3: Into<Xyz<T>> + From<Xyz<T>>
    {
        let viewport = viewport.cast(|p| T::from(p).unwrap(), |e| T::from(e).unwrap());
        let inverse = (proj * modelview).inverted();

        let two = T::one() + T::one();

        let mut tmp = Xyzw::point(ray.into());
        tmp.x = (tmp.x - viewport.x()) / viewport.w();
        tmp.y = (tmp.y - viewport.y()) / viewport.h();
        tmp = tmp * two - T::one();

        let mut obj = inverse * tmp;
        obj /= obj.w;

        obj.into_xyz().into()
    }
}




impl<T> From<Mat2<T>> for Mat3<T> where T: Zero + One {
    fn from(m: Mat2<T>) -> Self {
        let m = m.rows;
        Self {
            rows: Vec3(
                Vec3((m.0).0, (m.0).1, T::zero()),
                Vec3((m.1).0, (m.1).1, T::zero()),
                Vec3(T::zero(), T::zero(), T::one())
            )
        }
    }
}

impl<T> From<Mat3<T>> for Mat4<T> where T: Zero + One {
    fn from(m: Mat3<T>) -> Self {
        let m = m.rows;
        Self {
            rows: Vec4(
                Vec4((m.0).0, (m.0).1, (m.0).2, T::zero()),
                Vec4((m.1).0, (m.1).1, (m.1).2, T::zero()),
                Vec4((m.2).0, (m.2).1, (m.2).2, T::zero()),
                Vec4(T::zero(), T::zero(), T::zero(), T::one())
            )
        }
    }
}
impl<T> From<Mat2<T>> for Mat4<T> where T: Zero + One {
    fn from(m: Mat2<T>) -> Self {
        Self::from(Mat3::from(m))
    }
}
impl<T> From<Mat4<T>> for Mat3<T> where T: Zero + One {
    fn from(m: Mat4<T>) -> Self {
        let m = m.rows;
        Self {
            rows: Vec3(
                Vec3((m.0).0, (m.0).1, (m.0).2),
                Vec3((m.1).0, (m.1).1, (m.1).2),
                Vec3((m.2).0, (m.2).1, (m.2).2)
            )
        }
    }
}
impl<T> From<Mat3<T>> for Mat2<T> where T: Zero + One {
    fn from(m: Mat3<T>) -> Self {
        let m = m.rows;
        Self {
            rows: Vec2(
                Vec2((m.0).0, (m.0).1),
                Vec2((m.1).0, (m.1).1)
            )
        }
    }
}
impl<T> From<Mat4<T>> for Mat2<T> where T: Zero + One {
    fn from(m: Mat4<T>) -> Self {
        Self::from(Mat3::from(m))
    }
}
