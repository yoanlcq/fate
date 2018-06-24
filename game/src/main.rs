extern crate fate;
extern crate fate_gx as gx;
extern crate dmc;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate backtrace;

use std::time::{Duration, Instant};
use std::cell::RefCell;
use std::collections::VecDeque;
use fate::main_loop::{self, MainSystem, Tick, Draw};
use fate::lab::fps::{FpsManager, FpsCounter};
use fate::vek;
use vek::{Vec2, Extent2};
use gx::gl;

mod early;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Quit {
    DontCare,
    DontQuit,
    ShouldQuit,
    ForceQuit,
}

impl Default for Quit {
    fn default() -> Self {
        Quit::DontCare
    }
}

#[derive(Debug)]
struct FrameTimeManager {
    previous_frame_times: VecDeque<Duration>,
    current_frame_start: Option<Instant>,
    max_len: usize,
    average_frame_time: Duration,
    frame_time: Duration,
}

impl FrameTimeManager {
    pub fn with_max_len(max_len: usize) -> Self {
        Self {
            previous_frame_times: VecDeque::new(),
            current_frame_start: None,
            max_len,
            average_frame_time: Duration::default(),
            frame_time: Duration::default(),
        }
    }
    pub fn begin_main_loop_iteration(&mut self) {
        self.current_frame_start = Some(Instant::now());
    }
    pub fn end_main_loop_iteration  (&mut self) {
        self.previous_frame_times.push_back(self.current_frame_start.unwrap().elapsed());
        while self.previous_frame_times.len() > self.max_len {
            self.previous_frame_times.pop_front();
        }
        // Recompute average
        self.average_frame_time = {
            let mut sum = Duration::default();
            for d in self.previous_frame_times.iter() {
                sum += *d;
            }
            sum / self.previous_frame_times.len() as u32
        };
    }
    pub fn dt(&self) -> Duration {
        self.frame_time
    }
    pub fn smooth_dt(&self) -> Duration {
        self.average_frame_time
    }
}

#[derive(Debug)]
struct SharedGame {
    t: Duration, // Total physics time since the game started (accumulation of per-tick delta times)
    frame_time_manager: FrameTimeManager,
    pending_messages: VecDeque<Message>,
}
pub type G = SharedGame;

impl SharedGame {
    pub fn new() -> Self {
        Self {
            t: Duration::default(),
            frame_time_manager: FrameTimeManager::with_max_len(60),
            pending_messages: VecDeque::new(),
        }
    }
    pub fn push_message(&mut self, msg: Message) {
        self.pending_messages.push_back(msg);
    }
}

mod event {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Event {
        Quit,
        MouseMotion(i32, i32),
        CanvasResized(u32, u32),
        // Imagine, many other different kinds of event
    }

    impl Event {
        pub fn dispatch(&self, sys: &mut System, g: &mut G) {
            match *self {
                Event::Quit => sys.on_quit(g),
                Event::MouseMotion(x, y) => sys.on_mouse_motion(g, (x, y)),
                Event::CanvasResized(w, h) => sys.on_canvas_resized(g, (w, h)),
            }
        }
    }
}
use self::event::Event;


#[derive(Debug)]
pub enum Message {
    Foo,
    Bar,
}


// All items take &mut self since we know we're single-threaded.
trait System {
    fn quit(&self) -> Quit { Quit::DontCare }
    fn begin_main_loop_iteration(&mut self, _g: &mut G) {}
    fn end_main_loop_iteration  (&mut self, _g: &mut G) {}
    fn tick(&mut self, _g: &mut G, _t: &Tick) {}
    fn draw(&mut self, _g: &mut G, _d: &Draw) {}

    // messages
    fn on_message(&mut self, _g: &mut G, _msg: &Message) {}

    // events
    fn on_quit(&mut self, _g: &mut G) {}
    fn on_mouse_motion(&mut self, _g: &mut G, _pos: (i32, i32)) {}
    fn on_mouse_button(&mut self, _g: &mut G, _btn: u32, _is_down: bool) {}
    fn on_canvas_resized(&mut self, _g: &mut G, _size: (u32, u32)) {}
}


// --- Systems

struct ExampleSystem;
impl System for ExampleSystem {}

#[derive(Debug, Default)]
struct Quitter(Quit);
impl System for Quitter {
    fn quit(&self) -> Quit { self.0 }
    fn on_quit(&mut self, _: &mut G) { self.0 = Quit::ShouldQuit; }
}

#[derive(Debug, Default)]
struct ParticleSystemsState {
    pub positions: Vec<(f32, f32)>,
}
impl ParticleSystemsState {
    pub fn replace_by_lerp(&mut self, a: &Self, b: &Self, t: f32) {
    }
}
struct ParticleSystemsManager {
    states: [ParticleSystemsState; 2], // Doesn't have to be 2
    gfx_state: ParticleSystemsState, // Similarly, we coule have several ones.
    index_of_prev_state: usize,
    index_of_next_state: usize,
}

impl ParticleSystemsManager {
    pub fn new() -> Self {
        Self {
            states: [ParticleSystemsState::default(), ParticleSystemsState::default() ],
            gfx_state: ParticleSystemsState::default(),
            index_of_prev_state: 0,
            index_of_next_state: 1,
        }
    }
}
impl System for ParticleSystemsManager {
    fn tick(&mut self, _g: &mut G, _t: &Tick) {
        self.index_of_next_state += 1;
        self.index_of_next_state %= self.states.len();
        self.index_of_prev_state += 1;
        self.index_of_prev_state %= self.states.len();
    }
    fn draw(&mut self, g: &mut G, d: &Draw) {
        self.gfx_state.replace_by_lerp(&self.states[self.index_of_prev_state], &self.states[self.index_of_next_state], d.tick_progress as _);
    }
}


// TODO: Add systems at runtime
// Solved: Just don't; Know your systems ahead of time! Selectively enable/disable them at runtime.
// TODO: Retrieve a specific system at runtime
// Solved: It depends. Finding by key is annoying; Why not directly typing g.my_sys ? We know our game.


struct Game {
    dmc: dmc::Context,
    window: dmc::Window,
    gl_context: dmc::gl::GLContext,
    shared: RefCell<SharedGame>,
    systems: Vec<Box<System>>,
    fps_manager: FpsManager,
    fps_ceil: Option<f64>,
}

impl Game {
    pub fn new() -> Self {
        let shared = SharedGame::new();
        let mut systems = Vec::new();
        systems.push(Box::new(ExampleSystem) as Box<System>);
        systems.push(Box::new(Quitter::default()));
        systems.push(Box::new(ParticleSystemsManager::new()));
        let fps_manager = FpsManager {
            fps_counter: FpsCounter::with_interval(Duration::from_secs(1)),
            desired_fps_ceil: 64.,
            enable_fixing_broken_vsync: true,
        };
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
            profile: dmc::gl::GLProfile::Core,
            debug: true,
            forward_compatible: true,
            robust_access: None,
        };
        info!("GL pixel format settings: {:#?}", gl_pixel_format_settings);
        info!("GL context settings: {:#?}", gl_context_settings);
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
        gx::boot_gl();
        info!("{:#?}", gx::ContextSummary::new());
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

fn main() {
    early::setup_log();
    early::setup_panic_hook();
    early::setup_env();
    main_loop::run(&mut Game::new())
}

