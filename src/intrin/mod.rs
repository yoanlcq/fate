//! Intrinsics and related utilities.

pub mod likely;
pub use likely::*;
pub mod prefetch;
pub use prefetch::*;
pub mod debugtrap;
pub use debugtrap::*;
