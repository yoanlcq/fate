use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar, atomic::{self, AtomicBool, AtomicUsize}};
use taskbox::TaskBox;
use task::Task;
use Future;

pub type ThreadID = isize;

#[derive(Debug)]
pub struct SharedThreadContext {
    queue: Mutex<VecDeque<Arc<TaskBox>>>,
    should_threads_quit: AtomicBool,
    cvar: Condvar,
    status: Vec<AtomicUsize>,
}

#[derive(Debug, Clone)]
pub struct ThreadContext {
    pub id: ThreadID,
    pub shared: Arc<SharedThreadContext>,
}

impl SharedThreadContext {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            should_threads_quit: AtomicBool::new(false),
            cvar: Condvar::new(),
            status: (0 .. 32).map(|_| AtomicUsize::new(ThreadStatus::NONE as _)).collect(),
        }
    }
    pub fn store_quit(&self) {
        self.should_threads_quit.store(true, atomic::Ordering::SeqCst);
        self.cvar.notify_all();
    }
    fn should_threads_quit(&self) -> bool {
        atomic::spin_loop_hint();
        self.should_threads_quit.load(atomic::Ordering::SeqCst)
    }
    fn push_task(&self, task: Arc<TaskBox>) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_front(task);
        self.cvar.notify_one();
    }
    fn pop_task(&self) -> Option<Arc<TaskBox>> {
        let mut queue = self.queue.lock().unwrap();
        while let Some(task) = queue.pop_front() {
            if Arc::strong_count(&task) >= 2 {
                return Some(task);
            }
        }
        None
    }
    pub fn schedule<T: Task + Into<TaskBox>>(&self, t: T) -> Future<T> {
        let t = Arc::new(t.into());
        self.push_task(t.clone());
        unsafe { Future::new(t) }
    }
    pub fn thread_status(&self, id: ThreadID) -> Option<ThreadStatus> {
        ThreadStatus::try_from_usize(self.status[id as usize].load(atomic::Ordering::SeqCst))
    }
}

impl ThreadContext {
    fn set_status(&self, status: ThreadStatus) {
        self.shared.status[self.id as usize].store(status as u32 as usize, atomic::Ordering::SeqCst);
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ThreadStatus {
    Idle = 1,
    Polling = 2,
    Working = 3,
    Scheduling = 4,
}

impl ThreadStatus {
    pub const NONE: u32 = 0;
    pub fn try_from_usize(i: usize) -> Option<Self> {
        match i {
            i if i == ThreadStatus::Idle as u32 as usize => Some(ThreadStatus::Idle),
            i if i == ThreadStatus::Polling as u32 as usize => Some(ThreadStatus::Polling),
            i if i == ThreadStatus::Working as u32 as usize => Some(ThreadStatus::Working),
            i if i == ThreadStatus::Scheduling as u32 as usize => Some(ThreadStatus::Scheduling),
            _ => None,
        }
    }
}

pub fn thread_proc(cx: ThreadContext) {
    while !cx.shared.should_threads_quit() {
        cx.set_status(ThreadStatus::Polling);
        if let Some(task) = cx.shared.pop_task() {
            cx.set_status(ThreadStatus::Working);
            task.untyped_resume();
            cx.set_status(ThreadStatus::Scheduling);
            if !task.untyped_is_complete() {
                cx.shared.push_task(task);
            }
        } else {
            cx.set_status(ThreadStatus::Idle);
            let mut queue = cx.shared.queue.lock().unwrap();
            queue = cx.shared.cvar.wait(queue).unwrap();
            let _ = queue;
        }
    }
}

