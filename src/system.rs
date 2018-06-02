pub use std::time::Duration;
use global::G;
use std::any::Any;
use thread_mask::ThreadMask;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Tick {
    pub t: Duration,
    pub dt: Duration,
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Draw {
    pub progress_within_tick: f64,
}

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


// NOTE: Not all items take &self, because for now a `System` is guaranteed to be picked up by only
// one thread at a time. In the future we might introduce an other trait named `ConcurrentSystem`
// which always takes `&self`, such systems would be handled separately from regular `System`s.
pub trait System {
    //----- Threading and tasks

    fn thread_mask(&self) -> ThreadMask { ThreadMask::ONLY_MAIN_THREAD }
    /*
    fn thread_concurrency(&self, system_key: &str) -> usize { 1 }
    fn task_ordering(&self, system_key: &str) -> TaskOrdering {
        TaskOrdering::DONT_CARE // Or after, or before, depending on system_key
    }
    */


    //----- Main loop stuff

    /// Called every time the main loop begins a new iteration. Useful for FPS counters.
    fn begin_main_loop_iteration(&mut self, _g: &G) {}
    /// Called every time the main loop ends an iteration. Useful for FPS counters.
    fn end_main_loop_iteration  (&mut self, _g: &G) {}

    /// Replace previous state by current, and compute current state.
    fn tick(&mut self, _g: &G, _t: &Tick) {}

    /// Computes render state via interp, then renders.
    /// This takes `&mut self` because this is essentially "update for rendering, then render";
    /// Updating requires mutability and rendering should not.
    fn draw(&mut self, _g: &G, _d: &Draw) {}

    /// Asks this system if it agrees to quit the main loop.  
    ///
    /// The main loop exits when at least one system replies with `ForceQuit` OR 
    /// at least one system replies with `ShouldQuit` and no other system disagrees.
    fn quit(&self, _g: &G) -> Quit { Quit::DontCare }


    //----- Events/Messages

    /// Handles an event or message.
    /// Events are sent by the platform layer and are usually caused by human input.
    /// Messages are one way for systems to communicate asynchronously.
    fn event(&mut self, _g: &G, _ev: &Any) {}
}

// All of these take `&mut self` because there's always only one owner; That's the point.
// These also take `&G` but should not a priori deal with it; Anyway they can't do anything wrong
// with it since details are hidden beneath a safe, public API.
//
// poll_event returns &Any to avoid a Box allocation. This means that `self` must
// have some way of keeping the event alive (e.g as a struct member).
// Next time poll_event() is called, just replace that event by the new one.
// Systems won't be allowed to keep a reference to it; they have to clone it if they want to.
pub trait MainSystem {
    fn begin_main_loop_iteration(&mut self, g: &G);
    fn end_main_loop_iteration  (&mut self, g: &G);
    fn quit       (&mut self, g: &G) -> Quit;
    fn poll_event<'a>(&'a mut self, g: &G) -> Option<&'a Any>;
    fn before_tick(&mut self, g: &G, tick: &Tick);
    fn after_tick (&mut self, g: &G, tick: &Tick);
    fn before_draw(&mut self, g: &G, draw: &Draw);
    fn after_draw (&mut self, g: &G, draw: &Draw);
}

