use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use {SharedThreadContext, ThreadContext, ThreadID, thread_proc};

#[derive(Debug)]
pub struct ThreadPool {
    shared: Arc<SharedThreadContext>,
    map: HashMap<ThreadID, thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub fn with_capacity(shared: Arc<SharedThreadContext>, cap: usize) -> Self {
        Self { shared, map: HashMap::with_capacity(cap) }
    }
}

pub fn spawn_threads(nb: usize) -> (Arc<SharedThreadContext>, ThreadPool) {
    let cx = Arc::new(SharedThreadContext::new());
    let mut pool = ThreadPool::with_capacity(cx.clone(), nb);
    for id in 1 .. (1 + nb as isize) {
        let cx = ThreadContext {
            id,
            shared: cx.clone(),
        };
        let t = thread::spawn(move || thread_proc(cx));
        pool.map.insert(id, t);
    }
    (cx, pool)
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shared.store_quit();
        for (id, t) in self.map.drain() {
            info!("Waiting for thread {} to complete...", id);
            match t.join() {
                Ok(val) => info!("Thread {} exited with {:?}.", id, val),
                Err(e) => error!("Thread {} panicked: {:?}.", id, e),
            }
        }
    }
}
