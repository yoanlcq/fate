use std::fmt::{self, Display, Formatter};
use std::time::{Duration, Instant};
use duration_ext::DurationExt;
use system::{Tick, Draw};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TimeManager {
    pub t: Duration,
    pub dt: Duration,
    pub current_time: Instant,
    pub accumulator: Duration,
    pub frame_time_ceil: Duration,
    pub frame_time: Duration,
    pub fps_ceil: Option<f64>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct FpsCounter {
    pub interval: Duration,
    frame_accum: u64,
    last_time: Instant,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct FpsStats {
    pub frame_accum: u64,
    pub interval: Duration,
}

#[derive(Debug)]
pub struct Ticks<'a> {
    time: &'a mut TimeManager,
}

impl<'a> Iterator for Ticks<'a> {
    type Item = Tick;
    fn next(&mut self) -> Option<Tick> {
        if self.time.accumulator < self.time.dt {
            None
        } else {
            let tick = Tick { t: self.time.t, dt: self.time.dt };
            self.time.t += self.time.dt;
            self.time.accumulator -= self.time.dt;
            Some(tick)
        }
    }
}

impl TimeManager {
    pub fn with_fixed_dt_and_frame_time_ceil(dt: Duration, frame_time_ceil: Duration) -> Self {
        Self {
            t: Duration::default(),
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
            progress_within_tick: self.accumulator.to_f64_seconds() / self.dt.to_f64_seconds(),
        }
    }
    pub fn end_main_loop_iteration(&mut self) {
        if let Some(fps_ceil) = self.fps_ceil {
            let a_frame = Duration::from_f64_seconds(1. / fps_ceil);
            let ftime = Instant::now() - self.current_time;
            trace!("Time: frame_time={}, max_frame_time={}", ftime.to_f64_seconds(), a_frame.to_f64_seconds());
            if ftime < a_frame {
                trace!("Time: Sleeping for {} seconds", (a_frame - ftime).to_f64_seconds());
                ::std::thread::sleep(a_frame - ftime);
            }
        }
    }
}


impl FpsCounter {
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            interval,
            frame_accum: 0,
            last_time: Instant::now(),
        }
    }
    pub fn add_frame(&mut self) {
        self.frame_accum += 1;
    }
    pub fn try_sampling_fps(&mut self) -> Option<FpsStats> {
        debug_assert_ne!(self.interval, Duration::default(), "Sampling over an empty interval will yield incorrect results!");
        if Instant::now().duration_since(self.last_time) < self.interval {
            return None;
        }
        let fps_stats = FpsStats {
            frame_accum: self.frame_accum,
            interval: self.interval,
        };
        self.last_time += self.interval;
        self.frame_accum = 0;
        Some(fps_stats)
    }
}

impl FpsStats {
    pub fn fps(&self) -> f64 {
        self.frame_accum as f64 / self.interval.to_f64_seconds()
    }
}

impl Display for FpsStats {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, concat!("{} frames under {} seconds = ",
            "{} milliseconds/frame = ",
            "{} FPS"), 
            self.frame_accum,
            self.interval.to_f64_seconds(),
            1000. / self.fps(),
            self.fps()
        )
    }
}

