use system::*;
use dmc::device::{MouseButton, Key, ButtonState};

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Quit,
    MouseMotion(f64, f64),
    MouseMotionRaw(f64, f64),
    MouseButtonPressed(MouseButton),
    MouseButtonReleased(MouseButton),
    MouseEnter,
    MouseLeave,
    KeyboardFocusGained,
    KeyboardFocusLost,
    CanvasResized(u32, u32),
    KeyboardKeyPressed(Key),
    KeyboardKeyReleased(Key),
    KeyboardTextChar(char),
    KeyboardKeyPressedRaw(Key),
    KeyboardKeyReleasedRaw(Key),
}

impl Event {
    pub fn dispatch(&self, sys: &mut System, g: &mut G) {
        match *self {
            Event::Quit => sys.on_quit(g),
            Event::MouseEnter => sys.on_mouse_enter(g),
            Event::MouseLeave => sys.on_mouse_leave(g),
            Event::KeyboardFocusGained => sys.on_keyboard_focus_gained(g),
            Event::KeyboardFocusLost => sys.on_keyboard_focus_lost(g),
            Event::MouseMotion(x, y) => sys.on_mouse_motion(g, Vec2 { x, y }),
            Event::MouseMotionRaw(x, y) => sys.on_mouse_motion_raw(g, Vec2 { x, y }),
            Event::MouseButtonPressed(btn) => sys.on_mouse_button(g, btn, ButtonState::Down),
            Event::MouseButtonReleased(btn) => sys.on_mouse_button(g, btn, ButtonState::Up),
            Event::CanvasResized(w, h) => sys.on_canvas_resized(g, Extent2 { w, h }),
            Event::KeyboardKeyPressed(key) => sys.on_key(g, key, ButtonState::Down),
            Event::KeyboardKeyReleased(key) => sys.on_key(g, key, ButtonState::Up),
            Event::KeyboardTextChar(char) => sys.on_text_char(g, char),
            Event::KeyboardKeyPressedRaw(key) => sys.on_key_raw(g, key, ButtonState::Down),
            Event::KeyboardKeyReleasedRaw(key) => sys.on_key_raw(g, key, ButtonState::Up),
        }
    }
}

