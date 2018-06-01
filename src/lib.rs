//! Fight And Travel (Game) Engine.
//!
//! [It started in C](https://github.com/yoanlcq/fate-c), it will end in Rust.

#![doc(html_root_url = "https://docs.rs/fate/0.1.0")]
//#![deny(missing_docs)]
#![doc(test(attr(deny(warnings))))]
#![feature(test)]


extern crate test;
#[macro_use]
extern crate bitflags;
extern crate vek;

pub use vek::*;

mod main_loop_try;
pub mod thread_mask;
pub use thread_mask::ThreadMask;
pub mod global;
pub use global::Global;
pub mod system;
pub use system::System;

pub fn foo() {
    main_loop_try::main_try()
}