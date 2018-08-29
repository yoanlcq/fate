use system::*;

#[derive(Debug)]
pub struct Gameplay;

impl Gameplay {
    pub fn new() -> Self {
        Gameplay
    }
}

impl System for Gameplay {
}
