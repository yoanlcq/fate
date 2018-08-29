use std::os::raw::c_void;
use super::{Platform, Settings};
use fate::math::Extent2;
use event::Event;
use mouse_cursor::MouseCursor;
use dmc;
use sdl2::{self, Sdl, EventPump};
use sdl2::event::{Event as Sdl2Event, WindowEvent};
use sdl2::video::{Window, GLContext};

pub struct Sdl2Platform {
    sdl2: Sdl,
    window: Window,
    #[allow(dead_code)]
    gl_context: GLContext,
    event_pump: EventPump,
}

impl Sdl2Platform {
    pub fn new(settings: &Settings) -> Self {
        let &Settings {
            ref title,
            canvas_size: Extent2 { w, h },
            gl_pixel_format_settings: dmc::gl::GLPixelFormatSettings {
                msaa: dmc::gl::GLMsaa { buffer_count: msaa_buffer_count, sample_count },
                depth_bits,
                stencil_bits,
                double_buffer,
                stereo,
                red_bits,
                green_bits,
                blue_bits,
                alpha_bits,
                accum_red_bits,
                accum_blue_bits,
                accum_green_bits,
                accum_alpha_bits,
                aux_buffers: _,
                transparent: _,
            },
            gl_context_settings: dmc::gl::GLContextSettings {
                version: dmc::gl::GLVersion { major, minor, variant: _ },
                profile,
                debug,
                forward_compatible,
                robust_access: _,
            },
        } = settings;

        let sdl2 = sdl2::init().unwrap();
        let video_subsystem = sdl2.video().unwrap();
        
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_version(major, minor);
        gl_attr.set_context_profile(match profile {
            dmc::gl::GLProfile::Core => sdl2::video::GLProfile::Core,
            dmc::gl::GLProfile::Compatibility => sdl2::video::GLProfile::Compatibility,
        });
        {
            let mut flags = gl_attr.set_context_flags();
            if debug {
                flags.debug();
            }
            if forward_compatible {
                flags.forward_compatible();
            }
            flags.set();
        }

        gl_attr.set_red_size(red_bits);
        gl_attr.set_green_size(green_bits);
        gl_attr.set_blue_size(blue_bits);
        gl_attr.set_alpha_size(alpha_bits);
        gl_attr.set_double_buffer(double_buffer);
        gl_attr.set_depth_size(depth_bits);
        gl_attr.set_stencil_size(stencil_bits);
        gl_attr.set_stereo(stereo);
        gl_attr.set_multisample_buffers(msaa_buffer_count as _);
        gl_attr.set_multisample_samples(sample_count as _);
        gl_attr.set_accum_red_size(accum_red_bits);
        gl_attr.set_accum_green_size(accum_green_bits);
        gl_attr.set_accum_blue_size(accum_blue_bits);
        gl_attr.set_accum_alpha_size(accum_alpha_bits);

        // NOTE: DO NOT actually set this! It causes window creation to fail for some reason.
        // gl_attr.set_accelerated_visual(true)

        // This one could be interesting someday
        // gl_attr.set_framebuffer_srgb_compatible(value: bool);

        let window = video_subsystem.window(title, w, h)
            .opengl()
            .position_centered()
            .resizable()
            .build()
            .expect("Could not create window");
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
    fn canvas_size(&self) -> Extent2<u32> {
        self.window.size().into()
    }
    fn gl_swap_buffers(&mut self) {
        self.window.gl_swap_window();
    }
    fn gl_get_proc_address(&self, proc: &str) -> *const c_void {
        self.sdl2.video().unwrap().gl_get_proc_address(proc) as *const _
    }
    fn set_mouse_cursor(&mut self, mouse_cursor: &MouseCursor) {
        unimplemented!{}
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
