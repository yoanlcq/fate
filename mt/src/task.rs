use std::any::Any;

/// A trait to be implemented by self-contained work items.
pub trait Task {
    /// Type representing the computation's progress. This is mainly intended for displaying to users.
    type Progress;
    /// The type of the final value returned by the computation.
    type Result;
    /// Asks this task to progress "a bit", where "a bit" depends on how the task is configured.
    /// This is expected to be somewhat expensive (perform a long-running computation or perform actual I/O...),
    /// but not too much.
    ///
    /// In essence, `resume()` partially completes the task, then _yields_ execution to the calling
    /// thread simply by returning.
    ///
    /// This doesn't take `&mut self` so as to avoid having to wrap the whole Task in a RefCell or Mutex.
    /// Instead, the task has to selectively use interior mutability for relevant state.
    fn resume(&self);
    /// Gets a specialized description of the current "progress" state of the task.
    ///
    /// This may be as simple as a `bool` but also contain extra information that could be useful
    /// for displaying.
    /// It's better to make it such that the execution of `resume()` and `progress()` are not
    /// mutually exclusive. A thread that calls `progress()` expects an immediate reply, and should
    /// not wait for a lock used by `execute`.
    fn progress(&self) -> Self::Progress;
    /// Is this task complete? The value must be consistent with the return value of `progress()`.
    ///
    /// This is a separate function because building a new `Progress` instance may be more costly;
    /// otherwise, the semantics are the same as `progress()`.
    ///
    /// This is mainly intended for use by task executors.
    fn is_complete(&self) -> bool;
    /// Gets the task's result. Semantically, this _consumes_ the task, which should then be
    /// dropped.
    ///
    /// The following invariants must be held by the caller (otherwise the
    /// implementation is free to panic):
    /// - is_complete() is true;
    /// - This method is only ever called once, because semantically, the result is moved out of
    ///   this object. Unfortunately, this method cannot take `self` to enforce this, because
    ///   otherwise it could not be made into a trait object.
    ///
    /// These invariants are normally enforced at compile-time by the higher-level APIs.
    fn result(&self) -> Self::Result;
}

/// A type-erased counterpart to `Task`. Anything that implements `Task` automatically implements
/// `UntypedTask`.
///
/// Methods start with untyped_ to avoid clashes with those of `Task`.
pub trait UntypedTask {
    fn untyped_resume(&self);
    fn untyped_is_complete(&self) -> bool;
    fn untyped_progress(&self) -> Box<Any>;
    fn untyped_result(&self) -> Box<Any>;
}

impl<T> UntypedTask for T where T: Task, T::Progress: 'static, T::Result: 'static {
    fn untyped_resume(&self)               { self.resume() }
    fn untyped_is_complete(&self) -> bool  { self.is_complete() }
    fn untyped_progress(&self) -> Box<Any> { Box::new(self.progress()) }
    fn untyped_result(&self) -> Box<Any>   { Box::new(self.result()) }
}


