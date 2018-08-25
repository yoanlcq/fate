use std::boxed::FnBox;
use {Then, Task};

pub trait TaskExt: Task {
    /// Returns a Task which result is the one of the last task, which requires the
    /// completion of the first task.
    /// This effectively "merges" two tasks into one.
    fn then<T, F>(self, f: F) -> Then<Self, T> where Self: Sized + Sync, T: Task + Sync, F: FnBox(Self::Result) -> T + Send + 'static {
        Then::new(self, f)
    }
}

impl<T: Task> TaskExt for T {}

