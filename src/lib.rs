//! Fight And Travel (Game) Engine.
//!
//! [It started in C](https://github.com/yoanlcq/fate-c), it will end in Rust.

#![doc(html_root_url = "https://docs.rs/fate/0.1.0")]
//#![deny(missing_docs)]
#![doc(test(attr(deny(warnings))))]
#![feature(test)]

#![feature(optin_builtin_traits)] // FIXME: Remove this: Used only for GrisuiDmcPlatform test

// TODO: Goals for today:
// - Multithreaded main loop;
// - Async Resource loader.

extern crate test;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
extern crate vek;

pub use vek::*;

pub mod duration_ext;
pub mod fps;
pub mod main_loop;
/*
pub mod global;
pub use global::G;
pub mod system;
pub use system::System;
pub mod thread_mask;
pub use thread_mask::ThreadMask;
*/

mod main_loop_trivial;
mod main_loop_sequential;

pub fn make_it_look_like_main_loop_try_is_used() {
    main_loop_trivial::main();
    main_loop_sequential::main();
}
