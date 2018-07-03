use std::os::raw::c_void;
use super::{Platform, Settings};
use event::Event;
use dmc;
use fate::vek::{Vec2, Extent2};

pub struct DmcPlatform {
    dmc: dmc::Context,
    window: dmc::Window,
    #[allow(dead_code)]
    gl_context: dmc::gl::GLContext,
}

impl DmcPlatform {
    pub fn new(settings: &Settings) -> Self {
        let &Settings {
            ref title,
            canvas_size,
            ref gl_pixel_format_settings,
            ref gl_context_settings,
        } = settings;

        let dmc = dmc::Context::new().unwrap();

        let window = dmc.create_window(&dmc::WindowSettings {
            high_dpi: false,
            opengl: Some(&dmc::gl::GLDefaultPixelFormatChooser::from(gl_pixel_format_settings)),
        }).unwrap();

        window.set_size(canvas_size).unwrap();
        window.set_title(title).unwrap();

        let gl_context = window.create_gl_context(gl_context_settings).unwrap();
        window.make_gl_context_current(Some(&gl_context)).unwrap();

        if let Err(_) = window.gl_set_swap_interval(dmc::gl::GLSwapInterval::LateSwapTearing) {
            let _ = window.gl_set_swap_interval(dmc::gl::GLSwapInterval::VSync);
        }

        Self {
            dmc, window, gl_context,
        }
    }
}

impl Platform for DmcPlatform {
    fn show_window(&mut self) {
        self.window.show().unwrap();
    }
    fn gl_swap_buffers(&mut self) {
        self.window.gl_swap_buffers().unwrap();
    }
    fn gl_get_proc_address(&self, proc: &str) -> *const c_void {
        self.gl_context.proc_address(proc)
    }
    fn poll_event(&mut self) -> Option<Event> {
        match self.dmc.poll_event()? {
            dmc::Event::Quit => Some(Event::Quit),
            dmc::Event::WindowCloseRequested { .. } => Some(Event::Quit),
            dmc::Event::MouseMotion { position: Vec2 { x, y }, .. } => Some(Event::MouseMotion(x as _, y as _)),
            dmc::Event::WindowResized { size: Extent2 { w, h }, .. } => Some(Event::CanvasResized(w, h)),
            _ => None,
        }
    }
}
