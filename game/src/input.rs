use std::collections::HashMap;
use std::mem;
use dmc::device::{MouseButton, ButtonState, Keysym, KeyState};
use fate::vek::{Vec2, Vec3};
use system::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Input {
	keys: HashMap<Keysym, KeyState>,
	mouse_buttons: HashMap<MouseButton, ButtonState>,
	mouse_position: Option<Vec2<f64>>,
	mouse_displacement: Vec2<f64>,
    is_mouse_inside: bool,
    has_keyboard_focus: bool,
    quit_requested: bool,
    previous_canvas_size: Extent2<u32>,
    canvas_size: Extent2<u32>,
}

impl Input {
    pub fn new(canvas_size: Extent2<u32>) -> Self {
        Self {
            previous_canvas_size: canvas_size,
            canvas_size,
            .. Self::default()
        }
    }
	pub fn key(&self, k: Keysym) -> KeyState {
		*self.keys.get(&k).unwrap_or(&ButtonState::Up)
	}
	pub fn mouse_button(&self, btn: MouseButton) -> ButtonState {
		*self.mouse_buttons.get(&btn).unwrap_or(&ButtonState::Up)
	}
	pub fn mouse_position(&self) -> Option<Vec2<f64>> {
		self.mouse_position
	}
    pub fn mouse_displacement(&self) -> Vec2<f64> {
        self.mouse_displacement
    }
    pub fn is_mouse_inside(&self) -> bool {
        self.is_mouse_inside
    }
    pub fn has_keyboard_focus(&self) -> bool {
        self.has_keyboard_focus
    }
    pub fn canvas_size(&self) -> Extent2<u32> {
        self.canvas_size
    }
    pub fn previous_canvas_size(&self) -> Extent2<u32> {
        self.previous_canvas_size
    }
    pub fn quit_requested(&self) -> bool {
        self.quit_requested
    }
    pub fn keyboard_dpad(&self, n: Vec3<Keysym>, p: Vec3<Keysym>) -> Vec3<i32> {
        let x = self.key(p.x).is_down() as i32 - self.key(n.x).is_down() as i32;
        let y = self.key(p.y).is_down() as i32 - self.key(n.y).is_down() as i32;
        let z = self.key(p.z).is_down() as i32 - self.key(n.z).is_down() as i32;
        Vec3 { x, y, z }
    }
    pub fn keyboard_dpad_normalized(&self, negative: Vec3<Keysym>, positive: Vec3<Keysym>) -> Vec3<f32> {
        let v = self.keyboard_dpad(negative, positive).map(|x| x as f32);
        // Avoid division by zero possibly caused by normalization
        if v == Vec3::zero() { v } else { v.normalized() }
    }
    pub fn debug_camera_keyboard_dpad(&self) -> Vec3<f32> {
        // XXX: Fine because I'm AZERTY, but it ain't flexible
        let n = Vec3::new(Keysym::Q, Keysym::A, Keysym::S);
        let p = Vec3::new(Keysym::D, Keysym::E, Keysym::Z);
        self.keyboard_dpad_normalized(n, p)
    }
}

pub struct InputUpdater;

impl InputUpdater {
    pub fn new() -> Self { InputUpdater }
}

impl System for InputUpdater {
    fn begin_main_loop_iteration(&mut self, g: &mut G) {
        g.input.mouse_displacement = Default::default();
    }
    fn on_quit(&mut self, g: &mut G) {
        g.input.quit_requested = true;
    }
    fn on_canvas_resized(&mut self, g: &mut G, size: Extent2<u32>) { 
        if size != g.input.canvas_size {
            g.input.previous_canvas_size = mem::replace(&mut g.input.canvas_size, size);
        }
    }
    fn on_mouse_enter(&mut self, g: &mut G) {
        g.input.is_mouse_inside = true;
    }
    fn on_mouse_leave(&mut self, g: &mut G) {
        g.input.is_mouse_inside = false;
        g.input.mouse_position = None;
        g.input.mouse_displacement = Default::default();
    }
    fn on_keyboard_focus_gained(&mut self, g: &mut G) {
        g.input.has_keyboard_focus = true;
    }
    fn on_keyboard_focus_lost(&mut self, g: &mut G) {
        g.input.has_keyboard_focus = false;
    }
    fn on_mouse_motion(&mut self, g: &mut G, pos: Vec2<f64>) {
        if g.input.mouse_position != Some(pos) {
            let previous_mouse_position = mem::replace(&mut g.input.mouse_position, Some(pos));
            if let Some(prev) = previous_mouse_position {
                g.input.mouse_displacement = pos - prev;
            }
        }
    }
    fn on_mouse_button(&mut self, g: &mut G, btn: MouseButton, state: ButtonState) {
        *g.input.mouse_buttons.entry(btn).or_insert(state) = state;
    }
    fn on_key(&mut self, g: &mut G, key: Key, state: KeyState) {
        if let Some(sym) = key.sym {
            *g.input.keys.entry(sym).or_insert(state) = state;
        }
    }
    fn on_mouse_motion_raw(&mut self, _g: &mut G, _displacement: Vec2<f64>) {}
    fn on_key_raw(&mut self, _g: &mut G, _key: Key, _state: KeyState) {}
    fn on_text_char(&mut self, _g: &mut G, _char: char) {}
}
