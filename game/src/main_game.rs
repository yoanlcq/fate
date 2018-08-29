use std::time::Duration;
use std::cell::RefCell;
use std::env;
use std::collections::VecDeque;

use fate::main_loop::{MainSystem, Tick as MainLoopTick, Draw as MainLoopDraw};
use fate::lab::duration_ext::DurationExt;
use fate::lab::fps::{FpsManager, FpsCounter};
use fate::mt;

use g::G;
use system::{System, Tick, Draw};
use platform::{self, Platform, DmcPlatform, Sdl2Platform};
use quit::{Quit, Quitter};
use input::InputUpdater;
use event::Event;
use r_gl45::{self, GLSystem};
use gpu::GpuEndFrame;
use gameplay::Gameplay;
use mouse_cursor::MouseCursor;
use viewport::ViewportInputHandler;


// Can't derive anything :/
pub struct MainGame {
    platform: Box<Platform>,
    mouse_cursor: MouseCursor,
    g: RefCell<G>,
    event_queue: VecDeque<Event>,
    systems: Vec<Box<System>>,
    fps_manager: FpsManager,
    fps_ceil: Option<f64>,
    #[allow(dead_code)]
    threads: mt::ThreadPool,
}

impl MainGame {
    pub fn new() -> Self {
        let platform_settings = platform::Settings::new();
        info!("Using GL pixel format settings: {:#?}", platform_settings.gl_pixel_format_settings);
        info!("Using GL context settings: {:#?}", platform_settings.gl_context_settings);

        let mut platform = match env::var("platform").as_ref().map(String::as_str) {
            Ok("sdl2") => Box::new(Sdl2Platform::new(&platform_settings)) as Box<Platform>,
            _ => Box::new(DmcPlatform::new(&platform_settings)) as Box<Platform>,
        };

        r_gl45::gl_setup::gl_setup(platform.as_ref());

        let canvas_size = platform.canvas_size();
        let (mt, threads) = mt::spawn_threads(3);
        let g = G::new(canvas_size, mt.clone());
        let systems: Vec<Box<System>> = vec![
            Box::new(InputUpdater::new()),
            Box::new(Quitter::default()),
            Box::new(ViewportInputHandler::new()),
            Box::new(Gameplay::new()),
            Box::new(GLSystem::new(canvas_size, &g)),
            Box::new(GpuEndFrame::new()),
        ];
        let fps_manager = FpsManager {
            fps_counter: FpsCounter::with_interval(Duration::from_secs(1)),
            desired_fps_ceil: 64.,
            enable_fixing_broken_vsync: true,
        };

        platform.show_window();
 
        Self {
            platform,
            mouse_cursor: MouseCursor::default(),
            g: RefCell::new(g),
            event_queue: VecDeque::with_capacity(2047),
            systems,
            fps_manager,
            fps_ceil: None,
            threads,
        }
    }
    pub fn poll_event(&mut self) -> Option<Event> {
        let ev = self.platform.poll_event();
        /*
        if let Some(ref ev) = ev {
            debug!("GAME EVENT: {:?}", ev);
        }
        */
        ev
    }
    pub fn pump_messages(&mut self) {
        while let Some(msg) = self.g.borrow_mut().pending_messages.pop_front() {
            for sys in self.systems.iter_mut() {
                sys.on_message(&mut self.g.borrow_mut(), &msg);
            }
        }
    }
}
impl MainSystem for MainGame {
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
        let mut g = self.g.borrow_mut();
        g.frame_time_manager.begin_main_loop_iteration();
        for sys in self.systems.iter_mut() {
            sys.begin_main_loop_iteration(&mut g);
        }
    }
    fn end_main_loop_iteration  (&mut self) {
        let mut g = self.g.borrow_mut();
        for sys in self.systems.iter_mut() {
            sys.end_main_loop_iteration(&mut g);
        }
        g.frame_time_manager.end_main_loop_iteration();
        let fps_stats = self.fps_manager.end_main_loop_iteration(&mut self.fps_ceil);
        if let Some(fps_stats) = fps_stats {
            g.push_fps_stats(fps_stats);
            // info!("{}", fps_stats);
        }
    }
    fn pump_events(&mut self) {
        self.pump_messages();
        while let Some(ev) = self.poll_event() {
            self.event_queue.push_back(ev);
        }
        while let Some(ev) = self.event_queue.pop_front() {
            for sys in self.systems.iter_mut() {
                ev.dispatch(sys.as_mut(), &mut self.g.borrow_mut());
            }
            self.pump_messages();
        }
    } 
    fn tick(&mut self, tick: &MainLoopTick) {
        let mut g = self.g.borrow_mut();
        g.t += tick.dt;

        let dt_as_duration = tick.dt;
        let tick = Tick {
            t: g.t,
            dt_as_duration,
            dt: dt_as_duration.to_f64_seconds() as _,
        };

        for sys in self.systems.iter_mut() {
            sys.tick(&mut g, &tick);
        }
    }
    fn draw(&mut self, draw: &MainLoopDraw) {
        let mut g = self.g.borrow_mut();

        let dt_as_duration = g.frame_time_manager.dt();
        let smooth_dt_as_duration = g.frame_time_manager.smooth_dt();
        let draw = Draw {
            t: g.t,
            dt_as_duration,
            smooth_dt_as_duration,
            dt: dt_as_duration.to_f64_seconds() as _,
            smooth_dt: smooth_dt_as_duration.to_f64_seconds() as _,
            tick_progress: draw.tick_progress,
        };

        if self.mouse_cursor != g.mouse_cursor {
            self.mouse_cursor = g.mouse_cursor;
            self.platform.set_mouse_cursor(&g.mouse_cursor);
        }

        for sys in self.systems.iter_mut() {
            sys.draw(&mut g, &draw);
        }
        self.platform.gl_swap_buffers();
    }
}

