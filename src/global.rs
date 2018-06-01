use std::thread;
use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use std::borrow::Cow;
use thread_mask::ThreadMask;
use system::System;

type CowStr = Cow<'static, str>;

#[derive(Default)]
pub struct Global {
    pub threads: RwLock<HashMap<CowStr, thread::JoinHandle<()>>>,
}

impl Global {
    pub fn new() -> Self {
        Self {
            threads: RwLock::new(HashMap::new()),
        }
    }
/*
    // Spawn threads    
    let g = Arc::new(g);
    for i in 1..6 {
        g.threads.write().unwrap().push({
            let g = g.clone();
            println!("Main: Thread {}: Spawning", i);
            thread::Builder::new()
                .name(format!("Thread {}", i))
                .spawn(move || g.thread_proc(i))
                .unwrap()
        });
    }
    g.thread_proc(0); // Process main thread

    // Join threads
    let mut t = g.threads.write().unwrap();
    for (i, t) in t.drain(..).enumerate() {
        t.join().unwrap();
        println!("Main: Thread {}: Joined", i+1);
    }
    */

    fn thread_proc(&self) {
        unimplemented!()
    }

    pub fn spawn_threads<T: Into<CowStr>>(&self, entries: &[(T, ThreadMask)]) {
        for entry in entries {
            let cowstr = entry.0.into();
            self.threads.write().unwrap().insert(cowstr, {
                println!("Main: Spawning thread `{}`", cowstr);
                thread::Builder::new()
                    .name(cowstr.into())
                    .spawn(move || self.thread_proc())
                    .unwrap()
            });
        }
    }
    pub fn register_systems(&self, systems: &[&System]) {
        unimplemented!()
    }
    pub fn run(&mut self) { // Takes mut because this should only be called once and in one place.
        unimplemented!()
    }
}

