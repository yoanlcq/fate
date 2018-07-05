use std::time::{Duration, Instant};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct FrameTimeManager {
    previous_frame_times: VecDeque<Duration>,
    current_frame_start: Instant,
    max_len: usize,
    average_frame_time: Duration,
}

impl FrameTimeManager {
    pub fn with_max_len(max_len: usize) -> Self {
        assert_ne!(max_len, 0);
        Self {
            previous_frame_times: VecDeque::new(),
            current_frame_start: Instant::now(), // Should actually be in begin_main_loop_iteration()
            max_len,
            average_frame_time: Duration::default(),
        }
    }
    pub fn begin_main_loop_iteration(&mut self) {
        self.current_frame_start = Instant::now();
    }
    pub fn end_main_loop_iteration  (&mut self) {
        let current_frame_end = Instant::now();
        self.previous_frame_times.push_back(current_frame_end - self.current_frame_start);
        self.current_frame_start = current_frame_end;

        while self.previous_frame_times.len() > self.max_len {
            self.previous_frame_times.pop_front();
        }
        // Recompute average
        self.average_frame_time = {
            let mut sum = Duration::default();
            for d in self.previous_frame_times.iter() {
                sum += *d;
            }
            sum / self.previous_frame_times.len() as u32
        };
    }
    pub fn dt(&self) -> Duration {
        self.previous_frame_times.back().map(Clone::clone).unwrap_or_default()
    }
    pub fn smooth_dt(&self) -> Duration {
        self.average_frame_time
    }
}

