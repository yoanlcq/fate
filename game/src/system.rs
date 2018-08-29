// TODO: Add systems at runtime
// Solved: Just don't; Know your systems ahead of time! Selectively enable/disable them at runtime.
// TODO: Retrieve a specific system at runtime
// Solved: It depends. Finding by key is annoying; Why not directly typing g.my_sys ? We know our game.

pub use std::time::Duration;
pub use fate::math::{Vec2, Extent2};
pub use fate::lab::duration_ext::DurationExt;
pub use dmc::device::{Key, KeyState, MouseButton, ButtonState, Keysym};
pub use quit::Quit;
pub use g::G;
pub use message::Message;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Tick {
    pub t: Duration,
    pub dt_as_duration: Duration,
    pub dt: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Draw {
    pub t: Duration,
    pub dt_as_duration: Duration,
    pub dt: f32,
    pub smooth_dt_as_duration: Duration,
    pub smooth_dt: f32,
    pub tick_progress: f64,
}

// All items take &mut self since we know we're single-threaded.
pub trait System {
    fn quit(&self) -> Quit { Quit::DontCare }
    fn begin_main_loop_iteration(&mut self, _g: &mut G) {}
    fn end_main_loop_iteration  (&mut self, _g: &mut G) {}
    fn tick(&mut self, _g: &mut G, _t: &Tick) {}
    fn draw(&mut self, _g: &mut G, _d: &Draw) {}

    // messages
    fn on_message(&mut self, _g: &mut G, _msg: &Message) {}

    // events
    fn on_quit(&mut self, _g: &mut G) {}
    fn on_canvas_resized(&mut self, _g: &mut G, _size: Extent2<u32>) {}
    fn on_mouse_enter(&mut self, _g: &mut G) {}
    fn on_mouse_leave(&mut self, _g: &mut G) {}
    fn on_keyboard_focus_gained(&mut self, _g: &mut G) {}
    fn on_keyboard_focus_lost(&mut self, _g: &mut G) {}
    fn on_mouse_motion(&mut self, _g: &mut G, _pos: Vec2<f64>) {}
    fn on_mouse_motion_raw(&mut self, _g: &mut G, _displacement: Vec2<f64>) {}
    fn on_mouse_scroll(&mut self, _g: &mut G, _delta: Vec2<f64>) {}
    fn on_mouse_scroll_raw(&mut self, _g: &mut G, _delta: Vec2<f64>) {}
    fn on_mouse_button(&mut self, _g: &mut G, _btn: MouseButton, _state: ButtonState) {}
    fn on_mouse_button_raw(&mut self, _g: &mut G, _btn: MouseButton, _state: ButtonState) {}
    fn on_key(&mut self, _g: &mut G, _key: Key, _state: KeyState) {}
    fn on_key_raw(&mut self, _g: &mut G, _key: Key, _state: KeyState) {}
    fn on_text_char(&mut self, _g: &mut G, _char: char) {}
}

