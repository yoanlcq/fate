use system::*;

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameCubemapArrayID {
    RGB8_1L_1x1 = 0,
    RGB8_1L_1024x1024 = 1,
}

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameTexture2DArrayID {
    RGB8_1L_1x1 = 0,
    RGB8_1L_1024x1024 = 1,
}

#[derive(Debug)]
pub struct Gameplay;

impl Gameplay {
    pub fn new() -> Self {
        Gameplay
    }
}

impl System for Gameplay {
}
