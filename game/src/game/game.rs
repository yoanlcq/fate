use std::time::Duration;
use std::cell::RefCell;
use std::env;
use std::thread;
use std::sync::{Arc, Mutex, atomic::{Ordering, AtomicBool}};
use std::collections::{VecDeque, HashMap};
use fate::main_loop::{MainSystem, Tick as MainLoopTick, Draw as MainLoopDraw};
use fate::lab::duration_ext::DurationExt;
use fate::lab::fps::{FpsManager, FpsCounter};
use fate::gx::{self, gl};
use super::SharedGame;
use async::{file_io::{self, LoadingFile}};
use scene::{SceneLogicSystem, SceneCommandClearerSystem};
use system::{System, Tick, Draw};
use platform::{self, Platform, DmcPlatform, Sdl2Platform};
use quit::{Quit, Quitter};
use input::InputUpdater;
use event::Event;
use gamegl::{gl_error_hook, GLSystem, gl_debug_message_callback};


// Can't derive anything :/
pub struct Game {
    platform: Box<Platform>,
    shared: RefCell<SharedGame>,
    event_queue: VecDeque<Event>,
    systems: Vec<Box<System>>,
    fps_manager: FpsManager,
    fps_ceil: Option<f64>,
    threads: HashMap<ThreadID, thread::JoinHandle<()>>,
    mt_shared: Arc<MtShared>,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ThreadID {
    pub name: String,
    pub i: usize,
}
#[derive(Debug)]
pub struct ThreadContext {
    pub id: ThreadID,
    pub mt_shared: Arc<MtShared>,
}
#[derive(Debug)]
pub struct MtShared {
    pub quit: AtomicBool,
    pub file_io_tasks_queue: Mutex<VecDeque<LoadingFile>>,
}

impl MtShared {
    pub fn new() -> Self {
        Self {
            quit: Default::default(),
            file_io_tasks_queue: Default::default(),
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        self.mt_shared.quit.store(true, Ordering::SeqCst);
        for (id, t) in self.threads.drain() {
            debug!("Waiting for thread `{}` to complete", id.name);
            t.join().unwrap();
        }
    }
}


impl Game {
    pub fn new() -> Self {
        let platform_settings = platform::Settings::new();
        info!("Using GL pixel format settings: {:#?}", platform_settings.gl_pixel_format_settings);
        info!("Using GL context settings: {:#?}", platform_settings.gl_context_settings);

        let mut platform = match env::var("platform").as_ref().map(String::as_str) {
            Ok("sdl2") => Box::new(Sdl2Platform::new(&platform_settings)) as Box<Platform>,
            _ => Box::new(DmcPlatform::new(&platform_settings)) as Box<Platform>,
        };

        gl::load_with(|s| {
            let f = platform.gl_get_proc_address(s);
            trace!("GL: {}: {}", if f.is_null() { "Failed" } else { "Loaded" }, s);
            f
        });
        info!("OpenGL context summary:\n{}", gx::ContextSummary::new());
        gx::set_error_hook(gl_error_hook);
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
        unsafe { gl::POST_HOOK = gl_post_hook; }
        gx::boot_gl();
        gx::set_debug_message_callback(Some(gl_debug_message_callback));
        gx::log_debug_message("OpenGL debug logging is enabled.");

        let canvas_size = platform.canvas_size();
        let shared = SharedGame::new(canvas_size);
        let systems: Vec<Box<System>> = vec![
            Box::new(InputUpdater::new()),
            Box::new(Quitter::default()),
            Box::new(SceneLogicSystem::new()),
            Box::new(GLSystem::new(canvas_size, &shared)),
            Box::new(SceneCommandClearerSystem::new()),
        ];
        let fps_manager = FpsManager {
            fps_counter: FpsCounter::with_interval(Duration::from_secs(1)),
            desired_fps_ceil: 64.,
            enable_fixing_broken_vsync: true,
        };

        let (mt_shared, threads) = {
            let mt_shared = Arc::new(MtShared::new());
            let mut threads = HashMap::new();
            for i in 1..4 {
                let id = ThreadID {
                    name: format!("Extra thread {}", i),
                    i,
                };
                let cx = ThreadContext {
                    id: id.clone(),
                    mt_shared: mt_shared.clone(),
                };
                debug!("Spawned thread `{}`", id.name);
                threads.insert(id, thread::spawn(move || thread_proc(cx)));
            }
            (mt_shared, threads)
        };

        platform.show_window();
 
        Self {
            platform,
            shared: RefCell::new(shared),
            event_queue: VecDeque::with_capacity(2047),
            systems,
            fps_manager,
            fps_ceil: None,
            mt_shared,
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
        let mut shared = self.shared.borrow_mut();
        shared.frame_time_manager.begin_main_loop_iteration();
        for sys in self.systems.iter_mut() {
            sys.begin_main_loop_iteration(&mut shared);
        }
    }
    fn end_main_loop_iteration  (&mut self) {
        let mut shared = self.shared.borrow_mut();
        for sys in self.systems.iter_mut() {
            sys.end_main_loop_iteration(&mut shared);
        }
        shared.frame_time_manager.end_main_loop_iteration();
        let fps_stats = self.fps_manager.end_main_loop_iteration(&mut self.fps_ceil);
        if let Some(fps_stats) = fps_stats {
            shared.push_fps_stats(fps_stats);
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
                ev.dispatch(sys.as_mut(), &mut self.shared.borrow_mut());
            }
            self.pump_messages();
        }
    } 
    fn tick(&mut self, tick: &MainLoopTick) {
        let mut shared = self.shared.borrow_mut();
        shared.t += tick.dt;

        let dt_as_duration = tick.dt;
        let tick = Tick {
            t: shared.t,
            dt_as_duration,
            dt: dt_as_duration.to_f64_seconds() as _,
        };

        for sys in self.systems.iter_mut() {
            sys.tick(&mut shared, &tick);
        }
    }
    fn draw(&mut self, draw: &MainLoopDraw) {
        let mut shared = self.shared.borrow_mut();

        let dt_as_duration = shared.frame_time_manager.dt();
        let smooth_dt_as_duration = shared.frame_time_manager.smooth_dt();
        let draw = Draw {
            t: shared.t,
            dt_as_duration,
            smooth_dt_as_duration,
            dt: dt_as_duration.to_f64_seconds() as _,
            smooth_dt: smooth_dt_as_duration.to_f64_seconds() as _,
            tick_progress: draw.tick_progress,
        };

        for sys in self.systems.iter_mut() {
            sys.draw(&mut shared, &draw);
        }
        self.platform.gl_swap_buffers();
    }
}


pub fn thread_proc(cx: ThreadContext) {
    while !cx.mt_shared.quit.load(Ordering::SeqCst) {
        let task = {
            let mut lock = cx.mt_shared.file_io_tasks_queue.lock().unwrap();
            lock.pop_front()
        };
        // FIXME: Sleep if there are no more tasks
        if let Some(task) = task {
            file_io::process_file_io_task(&cx, task);
        }
    }
}

