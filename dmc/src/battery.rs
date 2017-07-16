//! Getting the device's battery's state, if any

use Knowledge;
use std::time::Duration;

/// One of: Wired, FullyCharged, Charging and Discharging.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum BatteryStatus {
    /// Not running on a battery
    Wired,
    /// Battery is fully charged and not discharging
    FullyCharged,
    /// Battery is charging
    Charging,
    /// Battery is discharging
    Discharging,
}

/// The battery's state and useful associated information - Get it for the
/// user's device with the associated `query()` function.
/// 
/// It's all packed into a single struct because on most platforms, it's 
/// cheaper to query everything at once rather than "coming back" for 
/// individual pieces of information.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct BatteryState {
    #[allow(missing_docs)]
    pub status: Knowledge<BatteryStatus>,
    #[allow(missing_docs)]
    pub estimated_time_remaining: Knowledge<Duration>,
    /// The value is between 0 and 1 (both inclusive).
    pub percentage_remaining: Knowledge<f32>,
}

impl BatteryState {
    /// Query the user's device's battery's current state.
    /// 
    /// You should consider this operation as expensive, and the result is 
    /// not implicitly cached - it's up to you to cache it by yourself
    /// if you want to.
    pub fn query() -> Self {
        unimplemented!()
    }
}
