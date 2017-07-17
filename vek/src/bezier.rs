use vec::{Vec3, Vec4};
use geom::{Line2, Line3};

// TODO impl from/into vec4 and vec3, respectively
// TODO into_iter, iter_mut, etc (for concisely applying the same xform to all points)

macro_rules! bezier_impl_any {
    ($Bezier:ident $Point:ident) => {
        impl<T> $Bezier<T> {
            pub fn normalized_tangent(self, t: Progress) -> $Point<T> {
                self.evaluate_derivative(t).normalized()
            }
            // Approximates the curve's length by subdividing it into step_count+1 straight lines.
            pub fn approx_length(self, step_count: u32) {
	            let mut length = T::zero();
	            let mut prev_point = self.evaluate(T::zero());
                for i in 1..step_count+2 {
    		        let t = i/(step_count+T::one());
    		        let next_point = self.evaluate(t);
                    length += (next_point - prev_point).length();
    		        prev_point = next_point;
                }
	            length
            }
        }
    }
}

macro_rules! bezier_impl_quadratic {
    ($QuadraticBezier:ident $Point:ident $Line:ident) => {
        
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $QuadraticBezier<T>(pub $Point<T>, pub $Point<T>, pub $Point<T>);
        
        impl<T, Progress=f32> $QuadraticBezier<T> {
            pub fn evaluate(self, t: Progress) -> $Point<T> {
                self.0*(1-t)*(1-t) + self.1*2*(1-t)*t + self.2*t*t
            }
            pub fn evaluate_derivative(self, t: Progress) -> $Point<T> {
                let n = T::one() + T::one();
                (1-t)*n*(self.1-self.0) + t*n*(self.2-self.1)
            }
            pub fn from_line(line: $Line<T>) -> Self {
                $QuadraticBezier(line.a, line.a, line.b)
            }
		    // XXX not sure about the name
            /// Returns the constant matrix M such that,
            /// given `T = [1, t*t, t*t*t]` and `P` the vector of control points,
            /// `dot(T * M, P)` evalutes the Bezier curve at 't'.
	        pub fn matrix() -> Mat3<T> {
                Mat3 {
                    rows: Vec3(
                        Vec3( 1,  0, 0),
                        Vec3(-2,  2, 0),
                        Vec3( 1, -2, 1),
                    )
                }
            }
        }
        
        impl<T> From<Vec3<$Point<T>>> for $QuadraticBezier {
            fn from(v: Vec3<$Point<T>>) -> Self {
                $QuadraticBezier(v.0, v.1, v.2)
            }
        }
        impl<T> From<$QuadraticBezier> for Vec3<$Point<T>> {
            fn from(v: $QuadraticBezier) -> Self {
                Vec3(v.0, v.1, v.2)
            }
        }
        
        bezier_impl_any!($QuadraticBezier $Point)
    }
}

macro_rules! bezier_impl_cubic {
    ($CubicBezier:ident $Point:ident $Line:ident) => {
        
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $CubicBezier<T>(pub $Point<T>, pub $Point<T>, pub $Point<T>, pub $Point<T>);

        impl<T, Progress=f32> $CubicBezier<T> {
            pub fn evaluate(self, t: Progress) -> $Point<T> {
		        self.0*(1-t)*(1-t)*(1-t) + self.1*3*(1-t)*(1-t)*t + self.2*3*(1-t)*t*t + self.3*t*t*t
            }
            pub fn evaluate_derivative(self, t: Progress) -> $Point<T> {
        	    let n = T::one() + T::one() + T::one();
        		(1-t)*(1-t)*n*(self.1-self.0) + 2*(1-t)*t*n*(self.2-self.1) + t*t*n*(self.3-self.2)
        	}
            pub fn from_line(line: $Line<T>) -> Self {
                $CubicBezier(line.a, line.a, line.b, line.b)
            }
            // XXX not sure about the name
            /// Returns the constant matrix M such that,
            /// given `T = [1, t*t, t*t*t, t*t*t*t]` and `P` the vector of control points,
            /// `dot(T * M, P)` evalutes the Bezier curve at 't'.
	        pub fn matrix() -> Mat4<T> {
                Mat4 {
                    rows: Vec4(
                        Vec4( 1,  0,  0, 0),
                        Vec4(-3,  3,  0, 0),
                        Vec4( 3, -6,  3, 0),
                        Vec4(-1,  3, -3, 1),
                    )
                }
            }
            // pub fn circle(radius: T, curve_count: u32) ->
        }
        
        impl<T> From<Vec4<$Point<T>>> for $QuadraticBezier {
            fn from(v: Vec4<$Point<T>>) -> Self {
                $QuadraticBezier(v.0, v.1, v.2, v.3)
            }
        }
        impl<T> From<$QuadraticBezier> for Vec4<$Point<T>> {
            fn from(v: $QuadraticBezier) -> Self {
                Vec4(v.0, v.1, v.2, v.3)
            }
        }
        
        bezier_impl_any!($CubicBezier $Point)
    }
}

bezier_impl_quadratic!(QuadraticBezier2 Xy Line2);
bezier_impl_quadratic!(QuadraticBezier3 Xyz Line3);
bezier_impl_cubic!(CubicBezier2 Xy Line2);
bezier_impl_cubic!(CubicBezier3 Xyz Line3);
