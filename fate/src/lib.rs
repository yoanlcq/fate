#![doc(html_root_url = "https://docs.rs/fate/0.1.0")]
#![doc(test(attr(deny(warnings))))]
#![feature(use_extern_macros)]

pub extern crate fate_font as font;
pub extern crate fate_gx as gx;
pub use gx::check_gl;
pub extern crate fate_img as img;
pub extern crate fate_lab as lab;
pub extern crate fate_main_loop as main_loop;
pub extern crate fate_math as math;
pub extern crate fate_mt as mt;
