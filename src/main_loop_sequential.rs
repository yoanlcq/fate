use std::any::Any;
use std::time::Duration;
use std::cell::RefCell;
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

#[derive(Default)]
struct SharedGame {
    some_shared_data: (),
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
        let mut shared = SharedGame::default();
        let mut systems = Vec::new();
        systems.push(Box::new(ExampleSystem) as Box<System>);
        systems.push(Box::new(ExampleSystem));
        Self { shared: RefCell::new(shared), systems }
    }
}
impl MainSystem for Game {
    fn quit(&self) -> bool { false }

    fn fps_ceil(&self) -> Option<f64> { None }
    fn tick_dt(&self) -> Duration { Duration::from_millis(16) }
    fn frame_time_ceil(&self) -> Duration { Duration::from_millis(250) }

    fn begin_main_loop_iteration(&mut self) {}
    fn end_main_loop_iteration  (&mut self) {}

    fn pump_events(&mut self) {}
    fn tick(&mut self, _tick: &Tick) {}
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

