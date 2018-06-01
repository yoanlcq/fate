pub use std::time::Duration;
use global::Global;

pub trait System {
    fn name(&self) -> &str;
    /// Replace previous state by current, and compute current state.
    fn tick(&mut self, _g: &Global, _t: Duration, _dt: Duration) {}
    /// Computes render state via interp, then renders.
    fn draw(&mut self, _g: &Global, _progress_within_tick: f64) {}

    /*
    fn thread_mask(&self) -> ThreadMask { ThreadMask::ONLY_MAIN_THREAD }
    fn thread_concurrency(&self, system_key: &str) -> usize { 1 }
    fn task_ordering(&self, system_key: &str) -> TaskOrdering {
        TaskOrdering::DONT_CARE // Or after, or before, depending on system_key
    }
    // TODO: There should be a safe way to get the current game for the current thread, instead
    // of having to pass it around all the time
    fn on_quit_requested(&mut self, _g: &Game) {}
    fn on_text_input(&mut self, _g: &Game, _text: &str) {}
    fn on_key(&mut self, _g: &Game, _key: Key) {}
    fn on_mouse_scroll(&mut self, _g: &Game, _delta: Vec2<i32>) {}
    fn on_mouse_motion(&mut self, _g: &Game, _pos: Vec2<i32>) {}
    fn on_mouse_button(&mut self, _g: &Game, _btn: MouseButton) {}
    fn on_mouse_enter(&mut self, _g: &Game) {}
    fn on_mouse_leave(&mut self, _g: &Game) {}
    fn on_canvas_resized(&mut self, _g: &Game, _size: Extent2<u32>, _by_user: bool) {}

    fn on_message(&mut self, _g: &Game, _msg: &Message) {}
    */
}
