//! The Timeout enum, which is ither a fixed duration or infinite.

use std::time::Duration;

#[derive(Debug)]
pub enum Timeout {
    Set(Duration),
    Infinite,
}
impl From<Duration> for Timeout {
    fn from(d: Duration) -> Self {
        Timeout::Set(d)
    }
}

