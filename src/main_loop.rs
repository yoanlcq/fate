use std::time::{Duration, Instant};
use std::thread;
use duration_ext::DurationExt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Tick {
    pub dt: Duration,
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Draw {
    pub tick_progress: f64,
}

// Most of these take `&mut self` because there's always only one owner; That's the point.
pub trait MainSystem {
    fn quit(&self) -> bool;

    fn fps_ceil(&self) -> Option<f64>;
    fn tick_dt(&self) -> Duration;
    fn frame_time_ceil(&self) -> Duration;

    fn begin_main_loop_iteration(&mut self);
    fn end_main_loop_iteration  (&mut self);

    fn pump_events(&mut self);
    fn tick(&mut self, tick: &Tick);
    fn draw(&mut self, draw: &Draw);
}


pub fn run(m: &mut MainSystem) {
    if m.quit() {
        return;
    }
    let mut t = TimeManager::with_fixed_dt_and_frame_time_ceil(
        m.tick_dt(),
        m.frame_time_ceil(),
    );

    'main: loop {
        t.set_fps_ceil(m.fps_ceil());
        t.set_tick_dt(m.tick_dt());
        t.set_frame_time_ceil(m.frame_time_ceil());

        t.begin_main_loop_iteration();
        m.begin_main_loop_iteration();

        if m.quit() { break 'main; }
        m.pump_events();
        for tick in t.ticks() {
            if m.quit() { break 'main; }
            m.tick(&tick);
            if m.quit() { break 'main; }
            m.pump_events();
        }

        if m.quit() { break 'main; }
        m.draw(&t.draw());
        if m.quit() { break 'main; }

        m.end_main_loop_iteration();
        if m.quit() { break 'main; }
        t.end_main_loop_iteration();
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
struct TimeManager {
    pub dt: Duration,
    pub current_time: Instant,
    pub accumulator: Duration,
    pub frame_time_ceil: Duration,
    pub frame_time: Duration,
    pub fps_ceil: Option<f64>,
}

#[derive(Debug)]
struct Ticks<'a> {
    time: &'a mut TimeManager,
}

impl<'a> Iterator for Ticks<'a> {
    type Item = Tick;
    fn next(&mut self) -> Option<Tick> {
        if self.time.accumulator < self.time.dt {
            None
        } else {
            let tick = Tick { dt: self.time.dt };
            self.time.accumulator -= self.time.dt;
            Some(tick)
        }
    }
}

impl TimeManager {
    pub fn with_fixed_dt_and_frame_time_ceil(dt: Duration, frame_time_ceil: Duration) -> Self {
        Self {
            dt,
            current_time: Instant::now(),
            accumulator: Duration::default(),
            frame_time_ceil,
            frame_time: Duration::default(),
            fps_ceil: None,
        }
    }
    pub fn set_fps_ceil(&mut self, ceil: Option<f64>) {
        self.fps_ceil = ceil;
    }
    pub fn set_frame_time_ceil(&mut self, d: Duration) {
        self.frame_time_ceil = d;
    }
    pub fn set_tick_dt(&mut self, d: Duration) {
        self.dt = d;
    }
    pub fn begin_main_loop_iteration(&mut self) {
        let new_time = Instant::now();
        self.frame_time = new_time - self.current_time;
        self.current_time = new_time;
        self.accumulator += if self.frame_time > self.frame_time_ceil {
            self.frame_time_ceil
        } else {
            self.frame_time
        };
    }
    pub fn ticks(&mut self) -> Ticks {
        Ticks { time: self }
    }
    pub fn draw(&self) -> Draw {
        Draw {
            tick_progress: self.accumulator.to_f64_seconds() / self.dt.to_f64_seconds(),
        }
    }
    pub fn end_main_loop_iteration(&mut self) {
        if let Some(fps_ceil) = self.fps_ceil {
            let a_frame = Duration::from_f64_seconds(1. / fps_ceil);
            let ftime = Instant::now() - self.current_time;
            trace!("Time: frame_time={}, max_frame_time={}", ftime.to_f64_seconds(), a_frame.to_f64_seconds());
            if ftime < a_frame {
                trace!("Time: Sleeping for {} seconds", (a_frame - ftime).to_f64_seconds());
                thread::sleep(a_frame - ftime);
            }
        }
    }
}

