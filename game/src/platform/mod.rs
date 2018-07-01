use std::os::raw::c_void;
use event::Event;

pub mod sdl2_platform;
pub use self::sdl2_platform::Sdl2Platform;
pub mod dmc_platform;
pub use self::dmc_platform::DmcPlatform;

pub trait Platform {
    fn show_window(&mut self);
    fn gl_get_proc_address(&self, proc: &str) -> *const c_void;
    fn gl_swap_buffers(&mut self);
    fn poll_event(&mut self) -> Option<Event>;
}
