use std::os::raw::c_void;
use super::Platform;
use event::Event;
use sdl2::{self, Sdl, EventPump};
use sdl2::event::{Event as Sdl2Event, WindowEvent};
use sdl2::video::{Window, GLContext, GLProfile};

pub struct Sdl2Platform {
    sdl2: Sdl,
    window: Window,
    #[allow(dead_code)]
    gl_context: GLContext,
    event_pump: EventPump,
}

impl Sdl2Platform {
    pub fn new(w: u32, h: u32, title: &str) -> Self {
        let sdl2 = sdl2::init().unwrap();
        let video_subsystem = sdl2.video().unwrap();
        
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_version(4, 5);
        gl_attr.set_context_profile(GLProfile::Compatibility);

        let window = video_subsystem.window(title, w, h)
            .opengl()
            .position_centered()
            .resizable()
            .build()
            .unwrap();
        let gl_context = window.gl_create_context().unwrap();
        let event_pump = sdl2.event_pump().unwrap();

        Self {
            sdl2, window, gl_context, event_pump
        }
    }
}

impl Platform for Sdl2Platform {
    fn show_window(&mut self) {
        // Window starts shown
    }
    fn gl_swap_buffers(&mut self) {
        self.window.gl_swap_window();
    }
    fn gl_get_proc_address(&self, proc: &str) -> *const c_void {
        self.sdl2.video().unwrap().gl_get_proc_address(proc) as *const _
    }
    fn poll_event(&mut self) -> Option<Event> {
        match self.event_pump.poll_event()? {
            Sdl2Event::Quit {..} => Some(Event::Quit),
            Sdl2Event::MouseMotion { x, y, .. } => Some(Event::MouseMotion(x as _, y as _)),
            Sdl2Event::Window { win_event, .. } => match win_event {
                WindowEvent::Resized(w, h)
                | WindowEvent::SizeChanged(w, h) => Some(Event::CanvasResized(w as _, h as _)),
                _ => None,
            }
            _ => None,
        }
    }
}
