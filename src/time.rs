use std::fmt::{self, Display, Formatter};
use std::time::{Duration, Instant};
use duration_ext::DurationExt;

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
    pub fn pump_physics_steps<F>(&mut self, mut f: F) where F: FnMut(Duration, Duration) {
        while self.accumulator >= self.dt {
            f(self.t, self.dt);
            self.t += self.dt;
            self.accumulator -= self.dt;
        }
    }
    pub fn gfx_lerp_factor(&self) -> f64 {
        self.accumulator.to_f64_seconds() / self.dt.to_f64_seconds()
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

