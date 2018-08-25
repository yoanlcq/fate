use std::fmt::{self, Debug, Formatter};
use std::boxed::FnBox;

/// A `Box<FnBox() -> T + Send>` wrapper that implements `Debug`.
pub struct BoxFn<T>(Box<FnBox() -> T + Send>);

impl<T> Debug for BoxFn<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<BoxFn>")
    }
}

impl<T> BoxFn<T> {
    pub fn inner(self) -> Box<FnBox() -> T + Send> { self.0 }
    pub fn call(self) -> T { self.inner()() } 
}

impl<F, T> From<F> for BoxFn<T> where F: FnBox() -> T + Send + 'static {
    fn from(f: F) -> Self {
        BoxFn(Box::new(f))
    }
}


/// A `Box<FnBox() -> T + Send>` wrapper that implements `Debug`.
pub struct ArgBoxFn<A, T>(Box<FnBox(A) -> T + Send>);

impl<A, T> Debug for ArgBoxFn<A, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<ArgBoxFn>")
    }
}

impl<A, T> ArgBoxFn<A, T> {
    pub fn inner(self) -> Box<FnBox(A) -> T + Send> { self.0 }
    pub fn call(self, a: A) -> T { self.inner()(a) } 
}

impl<F, A, T> From<F> for ArgBoxFn<A, T> where F: FnBox(A) -> T + Send + 'static {
    fn from(f: F) -> Self {
        ArgBoxFn(Box::new(f))
    }
}
