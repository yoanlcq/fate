use std::sync::{Mutex, RwLock, atomic::{self, AtomicBool}};
use boxfn::ArgBoxFn;
use {Task, Either};

/// A combinator for chaining two tasks sequentially.
#[derive(Debug)]
pub struct Then<T: Task + Sync, E: Task + Sync> {
    first: T,
    last: RwLock<Option<E>>,
    f: Mutex<Option<ArgBoxFn<T::Result, E>>>,
    done: AtomicBool,
}

impl<T: Task + Sync, E: Task + Sync> Then<T, E> {
    pub fn new<F>(first: T, f: F) -> Self where F: Into<ArgBoxFn<T::Result, E>> {
        Self {
            first,
            last: RwLock::new(None),
            f: Mutex::new(Some(f.into())),
            done: AtomicBool::new(false),
        }
    }
    pub fn first(&self) -> &T {
        &self.first
    }
}

impl<T: Task + Sync, E: Task + Sync> Task for Then<T, E> {
    type Progress = Either<T::Progress, E::Progress>;
    type Result = E::Result;
    fn resume(&self) {
        {
            let last = self.last.read().unwrap();
            if let Some(last) = last.as_ref() {
                last.resume();
                if last.is_complete() {
                    self.done.store(true, atomic::Ordering::SeqCst);
                }
                return;
            }
        }

        self.first.resume();
        if self.first.is_complete() {
            if let Some(f) = self.f.lock().unwrap().take() {
                let result = self.first.result();
                *self.last.write().unwrap() = Some(f.call(result));
            }
        }
    }
    fn is_complete(&self) -> bool {
        self.done.load(atomic::Ordering::SeqCst)
    }
    fn progress(&self) -> Self::Progress {
        {
            let last = self.last.read().unwrap();
            if let Some(last) = last.as_ref() {
                return Either::Right(last.progress());
            }
        }
        Either::Left(self.first.progress())
    }
    fn result(&self) -> Self::Result {
        self.last.read().unwrap().as_ref().unwrap().result()
    }
}


