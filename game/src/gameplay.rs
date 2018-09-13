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

use fate::math::Rgba;
use viewport::ViewportNode;
use eid::EID;
use cubemap::{CubemapSelector, CubemapArrayID};
use texture2d::Texture2DArrayID;

pub fn init_g(g: &mut G) {
    g.cubemap_array_create(CubemapArrayID(GameCubemapArrayID::RGB8_1L_1x1 as _));
    g.cubemap_array_create(CubemapArrayID(GameCubemapArrayID::RGB8_1L_1024x1024 as _));

    g.texture2d_array_create(Texture2DArrayID(GameTexture2DArrayID::RGB8_1L_1x1 as _));
    g.texture2d_array_create(Texture2DArrayID(GameTexture2DArrayID::RGB8_1L_1024x1024 as _));

    // TODO: Upload textures
    // TODO: Initialize cameras
    // TODO: Call this function
}
