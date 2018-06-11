use std::time::Duration;

pub trait DurationExt {
    fn to_f64_seconds(&self) -> f64;
    fn from_f64_seconds(s: f64) -> Self;
}

impl DurationExt for Duration {
    fn to_f64_seconds(&self) -> f64 {
        self.as_secs() as f64 + (self.subsec_nanos() as f64 / 1_000_000_000_f64)
    }
    fn from_f64_seconds(s: f64) -> Self {
        let secs = s as u64;
        let nanos = (s - secs as f64) * 1_000_000_000.;
        Duration::new(secs, nanos as _)
    }
}
