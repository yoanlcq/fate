use fate::math::{Extent2};
use gpu::GpuTextureInternalFormat;

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct CubemapSelector {
    pub array_id: CubemapArrayID,
    pub cubemap: u16,
}

assert_eq_size!(cubemap_size; CubemapSelector, u32);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CubemapArrayID(pub u8);

impl CubemapArrayID {
    pub const MAX: usize = 32;
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum CubemapFace {
    PositiveX = 0,
    NegativeX = 1,
    PositiveY = 2,
    NegativeY = 3,
    PositiveZ = 4,
    NegativeZ = 5,
}

impl CubemapFace {
    pub fn try_from_terragen_suffix(&self, suffix: &str) -> Option<Self> {
        Some(match suffix {
            "ft" => CubemapFace::PositiveX,
            "bk" => CubemapFace::NegativeX,
            "up" => CubemapFace::PositiveY,
            "dn" => CubemapFace::NegativeY,
            "rt" => CubemapFace::PositiveZ,
            "lf" => CubemapFace::NegativeZ,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CubemapArrayInfo {
    // Parameters at creation
    pub levels: u32,
    pub internal_format: GpuTextureInternalFormat,
    pub size: Extent2<u32>,
    pub nb_cubemaps: u32,
}

impl CubemapArrayInfo {
    pub fn new() -> Self {
        Self {
            levels: 1,
            internal_format: GpuTextureInternalFormat::RGB8,
            size: Extent2::broadcast(1),
            nb_cubemaps: 0,
        }
    }
}

