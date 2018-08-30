use std::os::raw::c_void;
use fate::math::Extent2;
use dmc;
use event::Event;
use mouse_cursor::MouseCursor;

pub mod sdl2_platform;
pub use self::sdl2_platform::Sdl2Platform;
pub mod dmc_platform;
pub use self::dmc_platform::DmcPlatform;

pub trait Platform {
    fn canvas_size(&self) -> Extent2<u32>;
    fn show_window(&mut self);
    fn gl_get_proc_address(&self, proc: &str) -> *const c_void;
    fn gl_swap_buffers(&mut self);
    fn poll_event(&mut self) -> Option<Event>;
    fn set_mouse_cursor(&mut self, mouse_cursor: &MouseCursor);
    fn set_mouse_cursor_visible(&mut self, visible: bool);
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub title: String,
    pub canvas_size: Extent2<u32>,
    pub gl_pixel_format_settings: dmc::gl::GLPixelFormatSettings,
    pub gl_context_settings: dmc::gl::GLContextSettings,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            title: "Test Game".to_owned(),
            canvas_size: (800, 480).into(),
            gl_pixel_format_settings: dmc::gl::GLPixelFormatSettings {
                msaa: dmc::gl::GLMsaa { buffer_count: 1, sample_count: 4 },
                depth_bits: 24,
                stencil_bits: 8,
                double_buffer: true,
                stereo: false,
                red_bits: 8,
                green_bits: 8,
                blue_bits: 8,
                alpha_bits: 8,
                accum_red_bits: 0,
                accum_blue_bits: 0,
                accum_green_bits: 0,
                accum_alpha_bits: 0,
                aux_buffers: 0,
                transparent: false,
            },
            gl_context_settings: dmc::gl::GLContextSettings {
                version: dmc::gl::GLVersion::new_desktop(4, 5),
                profile: dmc::gl::GLProfile::Core,
                debug: true,
                forward_compatible: true,
                robust_access: None,
            },
        }
    }
}
