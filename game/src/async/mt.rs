use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::collections::{HashMap, VecDeque};
use std::thread;

pub type ThreadResult = Result<(), ()>;
pub type ThreadPool = HashMap<ThreadID, thread::JoinHandle<ThreadResult>>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ThreadID {
    pub name: String,
    pub i: usize,
}

#[derive(Debug)]
pub struct ThreadContext {
    pub id: ThreadID,
    pub mt_shared: Arc<MtShared>,
}

#[derive(Debug)]
pub struct MtShared {
    pub quit: AtomicBool,
    pub file_io_tasks_queue: Mutex<VecDeque<super::fs::LoadingFile>>,
}

impl MtShared {
    fn new() -> Self {
        Self {
            quit: Default::default(),
            file_io_tasks_queue: Default::default(),
        }
    }
}

pub fn spawn_threads(nb: usize) -> (Arc<MtShared>, ThreadPool) {
    let mt_shared = Arc::new(MtShared::new());
    let mut threads = HashMap::new();
    for i in 1 .. nb+1 {
        let id = ThreadID {
            name: format!("Extra thread {}", i),
            i,
        };
        let cx = ThreadContext {
            id: id.clone(),
            mt_shared: mt_shared.clone(),
        };
        debug!("Spawned thread `{}`", id.name);
        threads.insert(id, thread::spawn(move || thread_proc(cx)));
    }
    (mt_shared, threads)
}

fn thread_proc(cx: ThreadContext) -> ThreadResult {
    while !cx.mt_shared.quit.load(Ordering::SeqCst) {
        let task = {
            let mut lock = cx.mt_shared.file_io_tasks_queue.lock().unwrap();
            lock.pop_front()
        };
        // FIXME: Sleep if there are no more tasks
        if let Some(task) = task {
            super::fs::process_file_io_task(&cx, task);
        }
    }
    Ok(())
}



use super::{Loading, Progress};

impl MtShared {
    pub fn do_async<T, E, F>(&self, f: Box<F>) -> Async<T, E>
        where F: FnOnce() -> Result<T, E> + Send + 'static
    {
        let task_data = Arc::new(AsyncTaskData {
            thread_id: AtomicIsize::new(-1),
            is_complete: AtomicBool::new(false),
            f,
        });
        self.gp_tasks_queue.lock().unwrap().push_back(Async(task_data.clone()));
        Async(task_data)
    }
}


#[derive(Debug)]
struct AsyncTaskData<T, E> {
    pub thread_id: AtomicIsize,
    pub is_complete: AtomicBool,
    f: ????,
    pub result: Option<Result<T, E>>,
}

#[derive(Debug)]
pub struct Async<T, E>(Arc<AsyncTaskData>);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct AsyncProgress { is_complete: bool }

impl Progress for AsyncProgress {
    fn is_complete(&self) -> bool { self.is_complete }
}

impl<T, E> Loading for Async<T, E> {
    type Item = T;
    type Error = E;
    type Progress = AsyncProgress;
    fn poll(&self) -> Self::Progress {
        AsyncProgress {
            is_complete: self.0.is_complete.load(Ordering::SeqCst)
        }
    }
    fn wait(self) -> Result<Self::Item, Self::Error> {
        loop {
            match Arc::try_unwrap(self.0) {
                Err(arc) => self.0 = arc,
                Ok(data) => break data.result.unwrap(),
            }
            atomic::spin_loop_hint();
        }
    }
    fn cancel(self) { drop(self) }
}
