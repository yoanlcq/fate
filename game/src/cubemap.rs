use fate::math::{Extent2, Rgba};
use gpu::{CpuImg, GpuTextureInternalFormat};

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
    levels: u32,
    internal_format: GpuTextureInternalFormat,
    size: Extent2<u32>,
    nb_cubemaps: u32,
    // For the whole array
    clear_color: Rgba<u8>,
    // Cubemaps
    keep_in_cpu: bool,
    faces: Vec<[CpuImg; 6]>,
}

impl CubemapArrayInfo {
    pub fn new() -> Self {
        Self {
            levels: 1,
            internal_format: GpuTextureInternalFormat::RGB8,
            size: Extent2::broadcast(1),
            nb_cubemaps: 0,
            clear_color: Rgba::new_opaque(0.3, 0.8, 1.).map(|x| (x * 255.) as u8),
            keep_in_cpu: false,
            faces: Vec::new(),
        }
    }
}
