use global::Global;
use system::System;
use thread_mask::ThreadMask;

struct FooManager;

impl FooManager {
    pub fn new() -> Self {
        FooManager
    }
}
impl System for FooManager {
    fn name(&self) -> &str {
        "FooManager"
    }
}

struct BarManager;

impl BarManager {
    pub fn new(_foo_manager: &FooManager) -> Self {
        BarManager
    }
}
impl System for BarManager {
    fn name(&self) -> &str {
        "BarManager"
    }
}

// Custom threads. Store these for later retrieval
pub static IA_THREAD: &'static str = "IA Thread";
pub static MY_THREAD: &'static str = "My Thread";

pub fn main_try() {
    let mut g = Global::new();
    g.spawn_threads(&[
        (IA_THREAD, ThreadMask::ANY),
        (MY_THREAD, ThreadMask::ANY),
    ]);
    let foo_manager = FooManager::new();
    let bar_manager = BarManager::new(&foo_manager);
    g.register_systems(&[
        &foo_manager,
        &bar_manager,
    ]);
    g.run()
}

