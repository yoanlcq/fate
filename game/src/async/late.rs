use std::mem;
use super::Loading;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Late<T: Loading> { // Convenience enum for something that may or may not be loaded.
    Loading(T),
    Complete(Result<T::Item, T::Error>),
}
impl<T: Loading> From<T> for Late<T> {
    fn from(loading: T) -> Self {
        Self::new(loading)
    }
}
impl<T: Loading> From<Result<T::Item, T::Error>> for Late<T> {
    fn from(result: Result<T::Item, T::Error>) -> Self {
        Late::Complete(result)
    }
}
impl<T: Loading> Late<T> {
    pub fn new(loading: T) -> Self { Late::Loading(loading) }
    pub fn is_loading(&self) -> bool { if let Late::Loading(_) = *self { true } else { false } }
    pub fn is_loaded(&self) -> bool { if let Late::Complete(Ok(_)) = *self { true } else { false } }
    pub fn is_failed(&self) -> bool { if let Late::Complete(Err(_)) = *self { true } else { false } }
    pub fn poll(&self) -> Option<T::Progress> {
        match *self {
            Late::Loading(ref loading) => Some(loading.poll()),
            Late::Complete(_) => None,
        }
    }
    pub fn wait(self) -> Result<T::Item, T::Error> {
        match self {
            Late::Loading(loading) => loading.wait(),
            Late::Complete(result) => result,
        }
    }
    pub fn get_mut(&mut self) -> Result<&mut T::Item, &mut T::Error> {
        let new = match *self {
            Late::Loading(ref loading) => Late::Complete(unsafe { ::std::ptr::read(loading) }.wait()),
            Late::Complete(ref result) => Late::Complete(unsafe { ::std::ptr::read(result) }),
        };
        mem::forget(mem::replace(self, new));
        match *self {
            Late::Complete(ref mut result) => result.as_mut(),
            _ => unreachable!(),
        }
    }
}

