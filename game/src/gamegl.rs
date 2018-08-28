use fate::math::{Extent2};
use fate::gx::{self, gl};
use platform::Platform;
use system::*;


static mut NB_ERRORS: usize = 0;

fn gl_debug_message_callback(msg: &gx::DebugMessage) {
    match ::std::ffi::CString::new(msg.text) {
        Ok(cstr) => debug!("GL: {}", cstr.to_string_lossy()),
        Err(e) => debug!("GL (UTF-8 error): {}", e),
    };
}

fn gl_post_hook(name: &str) {
    if name == "GetError" {
        return;
    }
    trace!("gl{}()", name);
    if unsafe { gx::SHOULD_TEMPORARILY_IGNORE_ERRORS } {
        return;
    }
    check_gl!(name);
}

fn gl_error_hook(e: Option<gx::Error>, s: &str) {
    match e {
        Some(e) => {
            error!("GL error: {:?} ({})", e, s);
            unsafe { NB_ERRORS += 1; }
        },
        None => if unsafe { NB_ERRORS > 0 } {
            panic!("Encountered {} OpenGL errors.", unsafe { NB_ERRORS });
        }
    }
}


pub fn init_gl(platform: &Platform) {
    gl::load_with(|s| {
        let f = platform.gl_get_proc_address(s);
        trace!("GL: {}: {}", if f.is_null() { "Failed" } else { "Loaded" }, s);
        f
    });
    info!("OpenGL context summary:\n{}", gx::ContextSummary::new());
    gx::set_error_hook(gl_error_hook);
    unsafe { gl::POST_HOOK = gl_post_hook; }
    gx::boot_gl();
    gx::set_debug_message_callback(Some(gl_debug_message_callback));
    gx::log_debug_message("OpenGL debug logging is enabled.");
}


#[derive(Debug)]
pub struct GLSystem {
    viewport_size: Extent2<u32>,
}

impl GLSystem {
    pub fn new(viewport_size: Extent2<u32>, _g: &G) -> Self {
        Self { viewport_size }
    }
}

impl System for GLSystem {
    fn on_canvas_resized(&mut self, _g: &mut G, size: Extent2<u32>) {
        self.viewport_size = size;
    }
    fn draw(&mut self, _g: &mut G, _d: &Draw) {
        self.gl_clear();
    }
}

impl GLSystem {
    fn gl_clear(&self) {
        unsafe {
            let Extent2 { w, h } = self.viewport_size;
            gl::Viewport(0, 0, w as _, h as _);
            gl::ClearColor(1., 0., 1., 1.);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}
