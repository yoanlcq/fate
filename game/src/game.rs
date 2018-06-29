use std::time::Duration;
use std::collections::VecDeque;
use std::cell::RefCell;
use fate::main_loop::{MainSystem, Tick, Draw};
use fate::lab::fps::{FpsManager, FpsCounter};
use fate::vek::{Vec2, Extent2};
use dmc;
use gx::{self, gl};
use frame_time::FrameTimeManager;
use message::Message;
use scene::{Scene, SceneCommandClearerSystem};
use system::System;
use quit::{Quit, Quitter};
use event::Event;
use gamegl::{GLSystem, gl_debug_message_callback};

pub struct Game {
    dmc: dmc::Context,
    window: dmc::Window,
    #[allow(dead_code)]
    gl_context: dmc::gl::GLContext,
    shared: RefCell<SharedGame>,
    systems: Vec<Box<System>>,
    fps_manager: FpsManager,
    fps_ceil: Option<f64>,
}

#[derive(Debug)]
pub struct SharedGame {
    pub t: Duration, // Total physics time since the game started (accumulation of per-tick delta times)
    pub frame_time_manager: FrameTimeManager,
    pub pending_messages: VecDeque<Message>,
    pub scene: Scene,
}

pub type G = SharedGame;


impl SharedGame {
    pub fn new() -> Self {
        Self {
            t: Duration::default(),
            frame_time_manager: FrameTimeManager::with_max_len(60),
            pending_messages: VecDeque::new(),
            scene: Scene::new(),
        }
    }
    #[allow(dead_code)]
    pub fn push_message(&mut self, msg: Message) {
        self.pending_messages.push_back(msg);
    }
}


impl Game {
    pub fn new() -> Self {
        let gl_pixel_format_settings = dmc::gl::GLPixelFormatSettings {
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
        };
        let gl_context_settings = dmc::gl::GLContextSettings {
            version: dmc::gl::GLVersion::new_desktop(4, 5),
            profile: dmc::gl::GLProfile::Compatibility,
            debug: true,
            forward_compatible: true,
            robust_access: None,
        };
        info!("Using GL pixel format settings: {:#?}", gl_pixel_format_settings);
        info!("Using GL context settings: {:#?}", gl_context_settings);
        let dmc = dmc::Context::new().unwrap();
        let window = dmc.create_window(&dmc::WindowSettings {
            high_dpi: false,
            opengl: Some(&dmc::gl::GLDefaultPixelFormatChooser::from(&gl_pixel_format_settings)),
        }).unwrap();
        let gl_context = window.create_gl_context(&gl_context_settings).unwrap();
        window.make_gl_context_current(Some(&gl_context)).unwrap();
        if let Err(_) = window.gl_set_swap_interval(dmc::gl::GLSwapInterval::LateSwapTearing) {
            let _ = window.gl_set_swap_interval(dmc::gl::GLSwapInterval::VSync);
        }
        gl::load_with(|s| {
            let f = gl_context.proc_address(s);
            trace!("GL: {}: {}", if f.is_null() { "Failed" } else { "Loaded" }, s);
            f
        });
        info!("OpenGL context summary:\n{}", gx::ContextSummary::new());
        gx::boot_gl();
        gx::set_debug_message_callback(Some(gl_debug_message_callback));
        gx::log_debug_message("OpenGL debug logging is enabled.");

        let shared = SharedGame::new();
        let mut systems = Vec::new();
        systems.push(Box::new(Quitter::default()) as Box<System>);
        systems.push(Box::new(GLSystem::new()));
        systems.push(Box::new(SceneCommandClearerSystem::new()));
        let fps_manager = FpsManager {
            fps_counter: FpsCounter::with_interval(Duration::from_secs(1)),
            desired_fps_ceil: 64.,
            enable_fixing_broken_vsync: true,
        };
 
        window.set_size(Extent2::new(800, 600)).unwrap();
        window.set_title("Test Game").unwrap();
        window.show().unwrap();

        Self {
            dmc,
            window,
            gl_context,
            shared: RefCell::new(shared),
            systems,
            fps_manager,
            fps_ceil: None,
        }
    }
    pub fn poll_event(&mut self) -> Option<Event> {
        use dmc::Event as DmcEvent;
        match self.dmc.poll_event()? {
            DmcEvent::Quit => Some(Event::Quit),
            DmcEvent::WindowCloseRequested { .. } => Some(Event::Quit),
            DmcEvent::MouseMotion { position: Vec2 { x, y }, .. } => Some(Event::MouseMotion(x as _, y as _)),
            DmcEvent::WindowResized { size: Extent2 { w, h }, .. } => Some(Event::CanvasResized(w, h)),
            _ => None,
        }
    }
    pub fn pump_messages(&mut self) {
        while let Some(msg) = self.shared.borrow_mut().pending_messages.pop_front() {
            for sys in self.systems.iter_mut() {
                sys.on_message(&mut self.shared.borrow_mut(), &msg);
            }
        }
    }
}
impl MainSystem for Game {
    fn quit(&self) -> bool {
        let mut should_quit = 0;
        let mut dont_quit = 0;
        for sys in self.systems.iter() {
            match sys.quit() {
                Quit::ForceQuit => return true,
                Quit::ShouldQuit => should_quit += 1,
                Quit::DontQuit => dont_quit += 1,
                Quit::DontCare => (),
            };
        }
        should_quit > 0 && dont_quit == 0
    }

    fn fps_ceil(&self) -> Option<f64> { self.fps_ceil }
    fn tick_dt(&self) -> Duration { Duration::from_millis(16) }
    fn frame_time_ceil(&self) -> Duration { Duration::from_millis(250) }

    fn begin_main_loop_iteration(&mut self) {
        self.shared.borrow_mut().frame_time_manager.begin_main_loop_iteration();
        for sys in self.systems.iter_mut() {
            sys.begin_main_loop_iteration(&mut self.shared.borrow_mut());
        }
    }
    fn end_main_loop_iteration  (&mut self) {
        for sys in self.systems.iter_mut() {
            sys.end_main_loop_iteration(&mut self.shared.borrow_mut());
        }
        self.shared.borrow_mut().frame_time_manager.end_main_loop_iteration();
        let fps_stats = self.fps_manager.end_main_loop_iteration(&mut self.fps_ceil);
        if let Some(fps_stats) = fps_stats {
            println!("{}", fps_stats);
        }
    }
    fn pump_events(&mut self) {
        self.pump_messages();
        while let Some(ev) = self.poll_event() {
            for sys in self.systems.iter_mut() {
                ev.dispatch(sys.as_mut(), &mut self.shared.borrow_mut());
            }
            self.pump_messages();
        }
    } 
    fn tick(&mut self, tick: &Tick) {
        let mut shared = self.shared.borrow_mut();
        shared.t += tick.dt;
        for sys in self.systems.iter_mut() {
            sys.tick(&mut shared, tick);
        }
    }
    fn draw(&mut self, draw: &Draw) {
        unsafe {
            gl::ClearColor(1., 0., 1., 1.);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        for sys in self.systems.iter_mut() {
            sys.draw(&mut self.shared.borrow_mut(), draw);
        }
        self.window.gl_swap_buffers().unwrap();
    }
}

