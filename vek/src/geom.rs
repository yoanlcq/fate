// TODO add useful impls to this module (inclusing basic conversions from rect to vec pairs)

// NOTE: in this module, the type parameters <P,E> usually stand for Position and Extent.

extern crate num_traits;
use self::num_traits::NumCast;
use core::mem;

use mat::Mat2;
use vec::*;

#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Rect<P,E> {
    /// Commonly assumed to be the top-left corner, because it is the de facto standard for this kind of struct.
    pub position: Xy<P>,
    /// Extent, with Y axis going downwards.
    pub extent: Extent2<E>,
}
/// A `Rect` extended to 3D.
///
/// This would have been named `Box`, but it was "taken" by the standard library already.
///
/// You should probably use `Aabb` because it is less confusing.
///
/// Rect3 is only useful when using extra precise integer coordinates where `Aabb` would only
/// allow for representing half the possible values for the extent. 
#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Rect3<P,E> {
    /// Commonly assumed to be the top-left-near corner.
    pub position: Xyz<P>,
    /// Extent, with Y axis going downwards.
    pub extent: Extent3<E>,
}
#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Aabb<P,E> {
    pub center: Xyz<P>,
    pub half_extent: Extent3<E>,
}

// NOTE: Only implement axis-aligned primitives (a.k.a don't go on a rampage).
// 
// Don't write, e.g a "Disk in 3D-space" structure, because users would rather
// represent it with a (Disk, z, orientation) tuple or anything else that suits their particular needs.
//
// On the other hand, everybody agrees that a minimal "Disk" struct is a position+radius pair.
// (even if it's just expressed as a radius with no
// position, then fine, just use the radius as-is, without making it it a new struct).
// 
// Any other info, such as fill color, border thickness, etc. are just extras that users can
// put on top (see composition over inheritance, etc).

#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Disk<P,E> {
    pub center: Xy<P>,
    pub radius: E,
}

impl<P,E> Disk<P,E> {
    pub fn area(self) -> E { unimplemented!() }
    pub fn diameter(&self) -> E { self.radius + self.radius }
    pub fn collision(self, other: Self) -> Xy<P> { unimplemented!() }
    pub fn collision_with_point(self, p: Xy<P>) -> Xy<P> { unimplemented!() }
}

#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Sphere<P,E> {
    pub center: Xyz<P>,
    pub radius: E,
}
impl<P,E> Sphere<P,E> {
    pub fn surface_area(self) -> E { unimplemented!() } // 4*pi*r*r
    pub fn volume(self) -> E { unimplemented!() } // pi*r*r*r*4/3
    pub fn diameter(&self) -> E { self.radius + self.radius }
    pub fn collision(self, other: Self) -> Xyz<P> { unimplemented!() }
    pub fn collision_with_point(self, p: Xyz<P>) -> Xyz<P> { unimplemented!() }
}

#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Ellipsis<P,E> {
    pub center: Xy<P>,
    pub radius: Extent2<E>,
}
/// Nobody can possibly use this ???
#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Potato<P,E> {
    pub center: Xyz<P>,
    pub radius: Extent3<E>,
}

#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Line2<T> {
    pub a: Xy<T>,
    pub b: Xy<T>,
}
#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Line3<T> {
    pub a: Xyz<T>,
    pub b: Xyz<T>,
}


impl<P,E> Rect<P,E> {
    pub fn new(x: P, y: P, w: E, h: E) -> Self {
        Self { position: Xy { x, y }, extent: Extent2 { w, h } }
    }
    pub fn into_pair(self) -> (Xy<P>, Extent2<E>) {
        ( Xy { x: self.position.x, y: self.position.y }, Extent2 { w: self.extent.w, h: self.extent.h })
    }
    pub fn to_pair(&self) -> (Xy<P>, Extent2<E>) where P: Clone, E: Clone {
        let s = self.clone();
        ( Xy { x: s.position.x, y: s.position.y }, Extent2 { w: s.extent.w, h: s.extent.h })
    }
    // Might look silly, but it's actually better then the other way around, because
    // there is less loss of information. A rect is actually a position and extent.
    // Direct acces to their components is only a shortcut.
    pub fn x(self) -> P { self.position.x }
    pub fn y(self) -> P { self.position.y }
    pub fn w(self) -> E { self.extent.w   }
    pub fn h(self) -> E { self.extent.h   }
    pub fn cast<DP,DE>(self) -> Option<Rect<DP,DE>> 
        where P: NumCast, E: NumCast, DP: NumCast, DE: NumCast
    {
        let mut out: Rect<DP,DE> = unsafe { mem::uninitialized() };
        if let Some(p) = self.position.cast() { out.position = p; } else { return None; };
        if let Some(e) = self.extent  .cast() { out.extent   = e; } else { return None; };
        Some(out)
    }
    pub fn collision_vector(self, _other: Self) -> Xy<P> {
        unimplemented!()    
    }
    pub fn split_v(self, _from_left: E) -> (Self, Self) {
        unimplemented!()
    }
    pub fn split_h(self, _from_top: E) -> (Self, Self) {
        unimplemented!()
    }
    pub fn split(self, _from_topleft: Extent2<E>) -> Mat2<Self> {
        unimplemented!()
    }
}

impl<P,E> From<(Xy<P>, Extent2<E>)> for Rect<P,E> {
    fn from(t: (Xy<P>, Extent2<E>)) -> Self {
        let position = t.0;
        let extent = t.1;
        Self { position, extent }
    }
}
