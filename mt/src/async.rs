use std::sync::{Mutex, atomic::{self, AtomicBool}};
use boxfn::BoxFn;
use Task;

/// Convenience for creating simple jobs based on a function that accepts anything (via the closure's
/// capture) and returns anything (including the unit `()` type).
///
/// This can indeed be used to turn any synchronous computation into an asynchronous one, with the
/// progress value being just a boolean: done, or not done.
///
/// If your needs are more complex, just create you own type that implements `Task` instead.
#[derive(Debug)]
pub struct Async<T> {
    data: Mutex<AsyncData<T>>,
    done: AtomicBool,
}

#[derive(Debug)]
struct AsyncData<T> {
    f: Option<BoxFn<T>>,
    v: Option<T>,
}

impl<T> Async<T> {
    pub fn new<F>(f: F) -> Self where F: Into<BoxFn<T>> {
        Self { 
            data: Mutex::new(AsyncData { f: Some(f.into()), v: None, }),
            done: AtomicBool::new(false),
        }
    }
}

impl<T> Task for Async<T> {
    type Progress = bool;
    type Result = T;
    fn resume(&self) {
        if self.is_complete() {
            return;
        }
        let mut data = self.data.lock().unwrap();
        if let Some(f) = data.f.take() {
            data.v = Some(f.call());
            self.done.store(true, atomic::Ordering::SeqCst);
        }
    }
    fn is_complete(&self) -> bool {
        self.done.load(atomic::Ordering::SeqCst)
    }
    fn progress(&self) -> bool { 
        self.done.load(atomic::Ordering::SeqCst)
    }
    fn result(&self) -> T {
        assert!(self.is_complete());
        self.data.lock().unwrap().v.take().unwrap()
    }
}


