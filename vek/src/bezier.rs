use vec::{Vec3, Vec4};

// TODO impl from/into vec4 and vec3, respectively

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
    ($QuadraticBezier:ident $Point:ident) => {
        
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
        }
        
        bezier_impl_any!($QuadraticBezier $Point)
    }
}

macro_rules! bezier_impl_cubic {
    ($CubicBezier:ident $Point:ident) => {
        
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
        }
        
        bezier_impl_any!($CubicBezier $Point)
    }
}

bezier_impl_quadratic!(QuadraticBezier2 Xy);
bezier_impl_quadratic!(QuadraticBezier3 Xyz);
bezier_impl_cubic!(CubicBezier2 Xy);
bezier_impl_cubic!(CubicBezier3 Xyz);
