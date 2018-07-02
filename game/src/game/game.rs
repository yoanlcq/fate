use std::time::{Instant, Duration};
use std::cell::RefCell;
use std::env;
use std::collections::VecDeque;
use fate::main_loop::{MainSystem, Tick, Draw};
use fate::lab::fps::{FpsManager, FpsCounter};
use fate::lab::duration_ext::DurationExt;
use super::SharedGame;
use gx::{self, gl};
use scene::SceneCommandClearerSystem;
use system::System;
use platform::{Platform, DmcPlatform, Sdl2Platform};
use quit::{Quit, Quitter};
use event::Event;
use gamegl::{GLSystem, gl_debug_message_callback};



pub struct Game {
    platform: Box<Platform>,
    shared: RefCell<SharedGame>,
    event_queue: VecDeque<Event>,
    systems: Vec<Box<System>>,
    fps_manager: FpsManager,
    fps_ceil: Option<f64>,
}

impl Game {
    pub fn new() -> Self {
        let mut platform = match env::var("platform").as_ref().map(String::as_str) {
            Ok("sdl2") => Box::new(Sdl2Platform::new(800, 600, "Test Game")) as Box<Platform>,
            _ => Box::new(DmcPlatform::new(800, 600, "Test Game")) as Box<Platform>,
        };

        gl::load_with(|s| {
            let f = platform.gl_get_proc_address(s);
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

        platform.show_window();
 
        Self {
            platform,
            shared: RefCell::new(shared),
            event_queue: VecDeque::with_capacity(2047),
            systems,
            fps_manager,
            fps_ceil: None,
        }
    }
    pub fn poll_event(&mut self) -> Option<Event> {
        // let start = Instant::now();
        let ev = self.platform.poll_event();
        // debug!("poll_event: {}", start.elapsed().to_f64_seconds());
        ev
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
            self.event_queue.push_back(ev);
        }
        while let Some(ev) = self.event_queue.pop_front() {
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
        for sys in self.systems.iter_mut() {
            sys.draw(&mut self.shared.borrow_mut(), draw);
        }
        self.platform.gl_swap_buffers();
    }
}

