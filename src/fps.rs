use std::fmt::{self, Display, Formatter};
use std::time::{Duration, Instant};
use duration_ext::DurationExt;


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


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FpsManager {
    pub fps_counter: FpsCounter,
    pub desired_fps_ceil: f64,
    pub enable_fixing_broken_vsync: bool,
}

impl FpsManager {
    pub fn end_main_loop_iteration(&mut self, fps_ceil: &mut Option<f64>) -> Option<FpsStats> {
        self.fps_counter.add_frame();
        if let Some(stats) = self.fps_counter.try_sampling_fps() {
            trace!("Main: New FPS stats: {}", &stats);
            if stats.fps() > self.desired_fps_ceil && self.enable_fixing_broken_vsync {
                warn!("Main: Broken VSync detected; FPS ceil is now set to {}", self.desired_fps_ceil);
                *fps_ceil = Some(self.desired_fps_ceil);
            }
            Some(stats)
        } else {
            None
        }
    }
}

