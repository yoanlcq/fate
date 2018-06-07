use std::any::Any;
use std::time::{Duration, Instant};
use std::cell::RefCell;
use std::collections::VecDeque;
use main_loop::{self, MainSystem, Tick, Draw};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Quit {
    DontCare,
    DontQuit,
    ShouldQuit,
    ForceQuit,
}

impl Default for Quit {
    fn default() -> Self {
        Quit::DontCare
    }
}

#[derive(Debug)]
struct FrameTimeManager {
    previous_frame_times: VecDeque<Duration>,
    current_frame_start: Option<Instant>,
    max_len: usize, // TODO: Init this
    current_average_frame_time: Option<Duration>,
}

impl FrameTimeManager {
    pub fn begin_main_loop_iteration(&mut self) {
        self.current_frame_start = Some(Instant::now());
    }
    pub fn end_main_loop_iteration  (&mut self) {
        self.previous_frame_times.push_back(self.current_frame_start.unwrap().elapsed());
        while self.previous_frame_times.len() > self.max_len {
            self.previous_frame_times.pop_front();
        }
        // Recompute average
        let mut avg = Duration::default();
        for d in self.previous_frame_times.iter() {
            avg += *d;
        }
        avg /= self.previous_frame_times.len() as u32;
        self.current_average_frame_time = Some(avg);
    }
    pub fn dt(&self) -> Option<Duration> {
        self.current_average_frame_time
    }
}

#[derive(Default)]
struct SharedGame {
    t: Duration, // Total physics time since the game started (accumulation of per-tick delta times)
    frame_time_manager: FrameTimeManager,
}
pub type G = SharedGame;

// All items take &mut self since we know we're single-threaded.
trait System {
    fn quit(&self) -> Quit { Quit::DontCare }
    fn begin_main_loop_iteration(&mut self, _g: &mut G) {}
    fn end_main_loop_iteration  (&mut self, _g: &mut G) {}
    fn tick(&mut self, _g: &mut G, _t: &Tick) {}
    fn draw(&mut self, _g: &mut G, _d: &Draw) {}
    fn event(&mut self, _g: &mut G, _ev: &Any) {}
}


// --- Systems

struct ExampleSystem;
impl System for ExampleSystem {}



struct Game {
    shared: RefCell<SharedGame>,
    systems: Vec<Box<System>>,
}
impl !Send for Game {}
impl !Sync for Game {}

impl Game {
    pub fn new() -> Self {
        let shared = SharedGame::default();
        let mut systems = Vec::new();
        systems.push(Box::new(ExampleSystem) as Box<System>);
        systems.push(Box::new(ExampleSystem));
        Self { shared: RefCell::new(shared), systems, }
    }
}
impl MainSystem for Game {
    fn quit(&self) -> bool {
        let mut should_quit = 0;
        let mut dont_quit = 0;
        for sys in self.systems.iter() {
            match sys.quit() {
                Quit::ForceQuit => return true,
                Quit::ShouldQuit => should_quit += 1,
                Quit::DontQuit => dont_quit += 1,
                Quit::DontCare => (),
            };
        }
        should_quit > 0 && dont_quit == 0
    }

    fn fps_ceil(&self) -> Option<f64> { None } // TODO: FpsLimiter
    fn tick_dt(&self) -> Duration { Duration::from_millis(16) }
    fn frame_time_ceil(&self) -> Duration { Duration::from_millis(250) }

    fn begin_main_loop_iteration(&mut self) {
        self.shared.borrow_mut().frame_time_manager.begin_main_loop_iteration();
    }
    fn end_main_loop_iteration  (&mut self) {
        self.shared.borrow_mut().frame_time_manager.end_main_loop_iteration();
    }

    fn pump_events(&mut self) {} // TODO: Dispatch event (like e.g Sdl2EventHandler), + custom messages
    fn tick(&mut self, tick: &Tick) {
        let mut shared = self.shared.borrow_mut();
        shared.t += tick.dt;
        for sys in self.systems.iter_mut() {
            sys.tick(&mut shared, tick); // TODO: physics swap buffers trick
        }
    }
    fn draw(&mut self, draw: &Draw) {
        // glClear()...
        for sys in self.systems.iter_mut() {
            sys.draw(&mut self.shared.borrow_mut(), draw);
        }
        // swap buffers...
    }
}

pub fn main() {
    main_loop::run(&mut Game::new())
}

