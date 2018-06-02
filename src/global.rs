use std::thread;
use std::any::Any;
use std::sync::{Mutex, Arc};
use std::collections::{HashMap, VecDeque};
use std::borrow::Cow;
use std::ops::Deref;
use std::time::Duration;
use thread_mask::ThreadMask;
use system::{System};

pub(crate) type CowStr = Cow<'static, str>;
pub(crate) type SystemBox = Box<System + Send>;

pub struct G(Arc<SharedG>);

pub struct SharedG {
    threads: Mutex<HashMap<CowStr, thread::JoinHandle<()>>>,
    pub(crate) systems: Mutex<HashMap<CowStr, SystemBox>>, // NOTE: Always a Box so these can be added and removed at runtime.
    pub(crate) messages: Mutex<VecDeque<Box<Any + Send>>>,
    fps_ceil: Mutex<Option<f64>>,
    tick_dt: Mutex<Duration>,
    frame_time_ceil: Mutex<Duration>,
}

impl Drop for SharedG {
    fn drop(&mut self) {
        // Join threads
        let mut t = self.threads.lock().unwrap();
        for (i, t) in t.drain() {
            t.join().unwrap();
            println!("Main: Thread {}: Joined", i);
        }
    }
}

impl Deref for G {
    type Target = SharedG;
    fn deref(&self) -> &SharedG {
        &self.0
    }
}

impl G {
    pub fn new() -> Self {
        G(Arc::new(SharedG::new()))
    }
    pub fn spawn_threads<T: IntoIterator<Item=(CowStr, ThreadMask)>>(&self, entries: T) {
        for (key, mask) in entries {
            self.spawn_thread(key, mask);
        }
    }
    pub fn spawn_thread<T: Into<CowStr>>(&self, key: T, mask: ThreadMask) {
        let key = key.into();
        self.threads.lock().unwrap().insert(key.clone(), {
            println!("Main: Spawning thread `{}`", key);
            let g = G(self.0.clone());
            thread::Builder::new()
                .name(key.clone().into())
                .spawn(move || g.thread_proc(mask, key.into()))
                .unwrap()
        });
    }
    fn thread_proc(self, my_mask: ThreadMask, my_name: CowStr) {
        for (key, sys) in self.systems.lock().unwrap().iter() {
            if sys.thread_mask().intersects(my_mask) {
                println!("Thread `{}`: Processing `{}`", my_name, key);
                let _ = sys.quit(&self); // FIXME: Remove. Just ensure we can pass self
            }
        }
    }
}

impl SharedG {
    fn new() -> Self {
        Self {
            threads: Default::default(),
            systems: Default::default(),
            messages: Default::default(),
            fps_ceil: Mutex::new(Some(124.)),
            tick_dt: Mutex::new(Duration::from_millis(16)),
            frame_time_ceil: Mutex::new(Duration::from_millis(512)),
        }
    }
    pub fn set_fps_ceil(&self, fps_ceil: Option<f64>) {
        *self.fps_ceil.lock().unwrap() = fps_ceil;
    }
    pub fn fps_ceil(&self) -> Option<f64> {
        *self.fps_ceil.lock().unwrap()
    }
    pub fn set_tick_dt(&self, d: Duration) {
        *self.tick_dt.lock().unwrap() = d;
    }
    pub fn tick_dt(&self) -> Duration {
        *self.tick_dt.lock().unwrap()
    }
    pub fn set_frame_time_ceil(&self, d: Duration) {
        *self.frame_time_ceil.lock().unwrap() = d;
    }
    pub fn frame_time_ceil(&self) -> Duration {
        *self.frame_time_ceil.lock().unwrap()
    }

    pub fn set_thread_mask(&self, key: &str, mask: ThreadMask) {
        if let Some(thread) = self.threads.lock().unwrap().get(key) {
            unimplemented!{}; //TODO: TLS ?
        }
    }
    pub fn join_thread(&self, key: &str) {
        if let Some(thread) = self.threads.lock().unwrap().remove(key) {
            thread.join().unwrap();
        }
    }
    // FIXME: This can be called from anywhere. Do not lock the systems; lock a dedicated queue
    // instead and wait until next frame
    pub fn register_system(&self, key: CowStr, system: SystemBox) {
        self.systems.lock().unwrap().insert(key.into(), system);
    }
    // FIXME: This can be called from anywhere. Do not lock the systems; lock a dedicated queue
    // instead and wait until next frame
    pub fn register_systems<T: IntoIterator<Item=(CowStr, SystemBox)>>(&self, systems: T) {
        for (key, sys) in systems.into_iter() {
            self.register_system(key, sys);
        }
    }
    // FIXME: This can be called from anywhere. Do not lock the systems; lock a dedicated queue
    // instead and wait until next frame
    pub fn unregister_system(&self, key: &str) {
        self.systems.lock().unwrap().remove(key);
    }
}

