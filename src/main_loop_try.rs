use global::{G, SystemBox};
use system::{MainSystem, System, Draw, Tick, Quit};
use std::any::Any;
use thread_mask::ThreadMask;

struct FooManager;

impl FooManager {
    pub fn new() -> Self {
        FooManager
    }
}
impl System for FooManager {
    fn draw(&mut self, g: &G, _d: &Draw) {
        println!("Hello world from FooManager!");
        let key = "dumb test foo thread";
        g.spawn_thread(key, ThreadMask::empty());
        g.set_thread_mask(key, ThreadMask::empty());
        g.join_thread(key);
        let key = "dumb test other foo system";
        g.register_system(key.into(), Box::new(Self::new()));
        g.unregister_system(key);
    }
}

struct BarManager;

impl BarManager {
    pub fn new(_foo_manager: &FooManager) -> Self {
        BarManager
    }
}
impl System for BarManager {
    fn draw(&mut self, _g: &G, _d: &Draw) {
        println!("Hello world from BarManager!");
    }
    fn quit(&self, _g: &G) -> Quit {
        Quit::ShouldQuit
    }
}

// This system calls glClear() and renders everything by reading commands from various compatible
// systems.
pub struct GLES2Renderer;
impl System for GLES2Renderer {
    fn thread_mask(&self) -> ThreadMask { ThreadMask::ONLY_MAIN_THREAD }
    fn draw(&mut self, _g: &G, _draw: &Draw) { println!("glClear(...)"); }
}

use time::FpsCounter;
use std::time::Duration;

pub struct FpsManager {
    fps_counter: FpsCounter,
    desired_fps_ceil: f64,
    enable_fixing_broken_vsync: bool,
}

impl FpsManager {
    pub fn new() -> Self {
        FpsManager {
            fps_counter: FpsCounter::with_interval(Duration::from_millis(2000)),
            desired_fps_ceil: 64.,
            enable_fixing_broken_vsync: true,
        }
    }
}

impl System for FpsManager {
    fn thread_mask(&self) -> ThreadMask { ThreadMask::ONLY_MAIN_THREAD }
    fn end_main_loop_iteration(&mut self, g: &G) {
        self.fps_counter.add_frame();
        if let Some(stats) = self.fps_counter.try_sampling_fps() {
            trace!("Main: New FPS stats: {}", &stats);
            if stats.fps() > self.desired_fps_ceil && self.enable_fixing_broken_vsync {
                g.set_fps_ceil(Some(self.desired_fps_ceil));
                warn!("Main: Broken VSync detected; FPS ceil is now set to {}", self.desired_fps_ceil);
            }
        }
    }
}


#[derive(Default)]
pub struct GrisuiDmcPlatform {
    _dmc: (),
    _window: (),
}

impl !Send for GrisuiDmcPlatform {}
impl !Sync for GrisuiDmcPlatform {}

impl MainSystem for GrisuiDmcPlatform {
    fn quit       (&mut self, _g: &G) -> Quit { Quit::DontCare }
    fn begin_main_loop_iteration(&mut self, _g: &G) {}
    fn end_main_loop_iteration(&mut self, _g: &G) {}
    fn poll_event<'a>(&'a mut self, _g: &G) -> Option<&'a Any> { /* poll event.. */ None }
    fn before_tick(&mut self, _g: &G, _tick: &Tick) {}
    fn after_tick (&mut self, _g: &G, _tick: &Tick) {}
    fn before_draw(&mut self, _g: &G, _draw: &Draw) {}
    fn after_draw (&mut self, _g: &G, _draw: &Draw) { /* swap buffers... */ }
}
impl GrisuiDmcPlatform {
    pub fn new() -> Self {
        // Initialize DMC, create window, set_title("Grisui")...
        Self::default()
    }
}


// Custom threads. Store these for later retrieval
pub static IA_THREAD: &'static str = "IA Thread";
pub static MY_THREAD: &'static str = "My Thread";

pub static MAIN_FOO_MANAGER: &'static str = "Main Foo Manager";
pub static MAIN_BAR_MANAGER: &'static str = "Main Bar Manager";
pub static MAIN_FPS_MANAGER: &'static str = "Main FPS Manager";
pub static GLES2_RENDERER: &'static str = "GL ES 2 Renderer";

mod early {
    pub fn setup_panic_hook() { unimplemented!{} }
    pub fn setup_env() { unimplemented!{} }
    pub fn setup_log() { unimplemented!{} }
}

pub fn main_try() {
    early::setup_panic_hook();
    early::setup_env();
    early::setup_log();

    let mut dmc = GrisuiDmcPlatform::new();
    let g = G::new();
    g.set_tick_dt(Duration::from_millis(60));
    g.set_fps_ceil(Some(124.));
    g.set_frame_time_ceil(Duration::from_millis(512));
    g.spawn_threads(vec![
        (IA_THREAD.into(), ThreadMask::ANY),
        (MY_THREAD.into(), ThreadMask::ANY),
    ]);
    let foo_manager = FooManager::new();
    let bar_manager = BarManager::new(&foo_manager);
    g.register_systems(vec![
        (MAIN_FOO_MANAGER.into(), Box::new(foo_manager) as SystemBox),
        (MAIN_BAR_MANAGER.into(), Box::new(bar_manager)),
        (GLES2_RENDERER.into(), Box::new(GLES2Renderer)),
        (MAIN_FPS_MANAGER.into(), Box::new(FpsManager::new())),
    ]);
    ::main_loop::run(&mut dmc, g); // Drop g before dmc, ensuring all threads have finished
}

