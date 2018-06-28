// TODO: Add systems at runtime
// Solved: Just don't; Know your systems ahead of time! Selectively enable/disable them at runtime.
// TODO: Retrieve a specific system at runtime
// Solved: It depends. Finding by key is annoying; Why not directly typing g.my_sys ? We know our game.

pub use fate::main_loop::{Tick, Draw};
pub use quit::Quit;
pub use game::G;
pub use message::Message;

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
    fn on_mouse_motion(&mut self, _g: &mut G, _pos: (i32, i32)) {}
    fn on_mouse_button(&mut self, _g: &mut G, _btn: u32, _is_down: bool) {}
    fn on_canvas_resized(&mut self, _g: &mut G, _size: (u32, u32)) {}
}


