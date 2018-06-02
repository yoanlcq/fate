use std::time::Duration;
use std::iter;
use std::mem;
use system::{MainSystem, Tick, Draw, Quit};
use time::TimeManager;
use global::G;

// Notes:
// + If the task is completed it can get the result,
//   and if not it can safely remove the task from the
//   queue and go ahead and perform that task itself.
// + Pipelining

// Main thread-specific stuff; Everything that isn't `Send` (e.g platform context) goes here.
struct Main<'a> {
    m: &'a mut MainSystem,
    g: G,
}

impl<'a> Main<'a> {
    fn new(m: &'a mut MainSystem, g: G) -> Self {
        Self { m, g }
    }
    pub fn fps_ceil(&mut self) -> Option<f64> {
        self.g.fps_ceil()
    }
    pub fn tick_dt(&mut self) -> Duration {
        self.g.tick_dt()
    }
    pub fn frame_time_ceil(&mut self) -> Duration {
        self.g.frame_time_ceil()
    }

    pub fn should_quit(&mut self) -> bool {
        let mut dont_quit = 0;
        let mut should_quit = 0;

        let systems = self.g.systems.lock().unwrap();
        let main_vote = self.m.quit(&self.g);
        let sys_votes = systems.iter().map(|(key, sys)| sys.quit(&self.g));

        for vote in iter::once(main_vote).chain(sys_votes) {
            match vote {
                Quit::ForceQuit => return true,
                Quit::ShouldQuit => should_quit += 1,
                Quit::DontQuit => dont_quit += 1,
                Quit::DontCare => (),
            };
        }
        should_quit > 0 && dont_quit == 0
    }
    pub fn pump_events(&mut self) {
        loop {
            if let Some(event) = self.m.poll_event(&self.g) {
                for (key, sys) in self.g.systems.lock().unwrap().iter_mut() {
                    sys.event(&self.g, event);
                }
            } else {
                break;
            }
            self.pump_messages();
        }
        self.pump_messages();
    }
    fn pump_messages(&mut self) {
        // Handling messages can cause new messages to be emitted
        while !self.g.messages.lock().unwrap().is_empty() {
            // replace() here allows us not to borrow the message queue while iterating,
            // which allows systems to push messages to the queue while already handling messages.
            for msg in mem::replace(&mut *self.g.messages.lock().unwrap(), Default::default()) {
                for (key, sys) in self.g.systems.lock().unwrap().iter_mut() {
                    sys.event(&self.g, &msg);
                }
            }
        }
    }
    pub fn begin_main_loop_iteration(&mut self) {
        self.m.begin_main_loop_iteration(&self.g);
        for (key, sys) in self.g.systems.lock().unwrap().iter_mut() {
            sys.begin_main_loop_iteration(&self.g);
        }
    }
    pub fn end_main_loop_iteration(&mut self) {
        for (key, sys) in self.g.systems.lock().unwrap().iter_mut() {
            sys.end_main_loop_iteration(&self.g);
        }
        self.m.end_main_loop_iteration(&self.g);
    }
    pub fn tick(&mut self, tick: &Tick) {
        self.m.before_tick(&self.g, tick);
        for (key, sys) in self.g.systems.lock().unwrap().iter_mut() {
            sys.tick(&self.g, tick);
        }
        self.m.after_tick(&self.g, tick);
    }
    pub fn draw(&mut self, draw: &Draw) {
        self.m.before_draw(&self.g, draw);
        for (key, sys) in self.g.systems.lock().unwrap().iter_mut() {
            sys.draw(&self.g, draw);
        }
        self.m.after_draw(&self.g, draw);
    }
}

fn can_process_task(tasks: &[Task], me: ThreadContext) -> Option<TaskID> {
    // TODO: Which task should this thread pick up? This is yours to implement!
}


pub fn run(main_sys: &mut MainSystem, g: G) {
    let mut m = Main::new(main_sys, g);
    if m.should_quit() {
        return;
    }
    let mut time = TimeManager::with_fixed_dt_and_frame_time_ceil(
        m.tick_dt(),
        m.frame_time_ceil(),
    );

    'main: loop {
        time.set_fps_ceil(m.fps_ceil());
        time.set_tick_dt(m.tick_dt());
        time.set_frame_time_ceil(m.frame_time_ceil());
        time.begin_main_loop_iteration();
        m   .begin_main_loop_iteration();

        if m.should_quit() { break 'main; }
        m.pump_events();
        for tick in time.ticks() {
            if m.should_quit() { break 'main; }
            m.tick(&tick);
            if m.should_quit() { break 'main; }
            m.pump_events();
        }

        if m.should_quit() { break 'main; }
        m.draw(&time.draw());
        if m.should_quit() { break 'main; }

        m   .end_main_loop_iteration();
        if m.should_quit() { break 'main; }
        time.end_main_loop_iteration();
    }
}

