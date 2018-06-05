use std::time::Duration;
use main_loop::{self, MainSystem, Tick, Draw};

#[derive(Default)]
struct TrivialGame;
impl !Send for TrivialGame {}
impl !Sync for TrivialGame {}

impl MainSystem for TrivialGame {
    fn quit(&self) -> bool { false }

    fn fps_ceil(&self) -> Option<f64> { None }
    fn tick_dt(&self) -> Duration { Duration::from_millis(16) }
    fn frame_time_ceil(&self) -> Duration { Duration::from_millis(250) }

    fn begin_main_loop_iteration(&mut self) {}
    fn end_main_loop_iteration  (&mut self) {}

    fn pump_events(&mut self) {}
    fn tick(&mut self, _tick: &Tick) {}
    fn draw(&mut self, _draw: &Draw) {}
}

pub fn main() {
    main_loop::run(&mut TrivialGame::default())
}

