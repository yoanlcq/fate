use std::sync::Arc;
use std::marker::PhantomData;
use std::raw;
use std::mem;
use std::ops::Deref;
use taskbox::TaskBox;
use Task;

#[derive(Debug)]
pub struct Future<T: Task> {
    task: Arc<TaskBox>,
    _pd: PhantomData<T>,
}

impl<T: Task> Future<T> {
    // This is unsafe because The TaskBox must have been created from an instance of T.
    pub(crate) unsafe fn new(task: Arc<TaskBox>) -> Self {
        Self {
            task, _pd: PhantomData,
        }
    }
    pub fn is_complete(&self) -> bool {
        self.task.untyped_is_complete()
    }
    pub fn poll(&self) -> T::Progress where T::Progress: 'static {
        *self.task.untyped_progress().downcast().unwrap()
    }
    pub fn wait(self) -> T::Result where T::Result: 'static {
        while !self.task.untyped_is_complete() {
            self.task.untyped_resume();
        }
        *self.task.untyped_result().downcast().unwrap()
    }
    pub fn cancel(self) { 
        /* Just drop the value, decreasing refcount */
    }
}

impl<T: Task> AsRef<T> for Future<T> {
    fn as_ref(&self) -> &T {
        unsafe {
            let raw: raw::TraitObject = mem::transmute(self.task.deref().as_ref());
            &*(raw.data as *const T)
        }
    }
}

impl<T: Task> Deref for Future<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.as_ref()
    }
}
