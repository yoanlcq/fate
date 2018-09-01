use fate::math::{Extent2};
use gpu::GpuTextureInternalFormat;

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Texture2DArrayID(pub u8);

impl Texture2DArrayID {
    pub const MAX: usize = 32;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Texture2DArrayInfo {
    // Parameters at creation
    pub levels: u32,
    pub internal_format: GpuTextureInternalFormat,
    pub size: Extent2<u32>,
    pub nb_textures: u32,
}

impl Texture2DArrayInfo {
    pub fn new() -> Self {
        Self {
            levels: 1,
            internal_format: GpuTextureInternalFormat::RGB8,
            size: Extent2::broadcast(1),
            nb_textures: 0,
        }
    }
}

