#![feature(fnbox)]
#![feature(raw)]

#[macro_use]
extern crate log;
extern crate either;

pub use either::Either;

pub mod boxfn;
pub mod taskbox;
pub mod task;
pub mod task_ext;
pub mod then;
pub mod future;
pub mod async;
pub mod exec;
pub mod fs;
pub mod thread_pool;

pub use task::Task;
pub use task_ext::TaskExt;
pub use then::Then;
pub use future::Future;
pub use async::Async;
pub use fs::{ReadFile, FileProgress};
pub use exec::{ThreadID, SharedThreadContext, ThreadContext, thread_proc};
pub use thread_pool::{ThreadPool, spawn_threads};
