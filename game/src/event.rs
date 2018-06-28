use system::*;

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

