//! DMC - DirectMedia Crate
//! 
//! This is an attempt at an SDL2 rewrite in Rust. The end goal is to get
//! rid of the dependency on SDL2's DLL for Rust apps.
//!
//! TODO be able to provide WM_QUERYENDSESSION and WM_ENDSESSION events (Win32)- See [Shutting Down](https://msdn.microsoft.com/en-us/library/windows/desktop/aa376881(v=vs.85).aspx)

#![doc(html_root_url = "https://docs.rs/dmc/0.1.0")]
//#![feature(test)]
//#![warn(missing_docs)]
#![doc(test(attr(deny(warnings))))]
#![cfg_attr(feature="cargo-clippy", allow(doc_markdown))]

// TODO "log!" everything

pub mod semver;
pub use semver::Semver;
pub mod display;
pub use display::Display;
pub mod game_input_device;
pub use game_input_device::{GameInputDevice, Dpad, Minmax, SignedAxis, UnsignedAxis};
pub mod event;
pub use event::{EventQueue, Clipboard, TextInput};
pub mod battery;
pub use battery::{BatteryState, BatteryStatus};
pub mod timeout;
pub use timeout::Timeout;
pub mod option_alternatives;
pub use option_alternatives::Decision;
pub use option_alternatives::Knowledge;
pub use option_alternatives::Decision::*;
pub use option_alternatives::Knowledge::*;
pub mod vec;
pub use vec::*;
