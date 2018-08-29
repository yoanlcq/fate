use system::*;

#[derive(Debug)]
pub struct Gameplay;

impl Gameplay {
    pub fn new() -> Self {
        Gameplay
    }
}

impl System for Gameplay {
    fn on_key(&mut self, g: &mut G, key: Key, state: KeyState) {
        match key.sym {
            Some(Keysym::V) if state.is_down() => g.viewport_split_v(),
            Some(Keysym::H) if state.is_down() => g.viewport_split_h(),
            Some(Keysym::M) if state.is_down() => g.viewport_merge(),
            _ => {},
        }
    }
}
