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
    pub const MAX: usize = 16;
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
    pub const TERRAGEN_SUFFIXES: [&'static str; 6] = [ "ft", "bk", "up", "dn", "rt", "lf" ];
    pub fn try_from_terragen_suffix(suffix: &str) -> Option<Self> {
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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct CubemapArrayInfo {
    // Parameters at creation
    pub nb_levels: u32,
    pub internal_format: GpuTextureInternalFormat,
    pub size: Extent2<u32>,
    pub nb_cubemaps: u32,
}

impl CubemapArrayInfo {
    pub fn memory_usage(&self) -> usize {
        let mut sum = 0;
        for level in 0 .. self.nb_levels {
            sum += self.memory_usage_at_level(level);
        }
        sum
    }
    pub fn memory_usage_at_level(&self, level: u32) -> usize {
        assert!(level < self.nb_levels);
        let size = self.size.map(|x| (x >> level) as usize);
        let bits = self.nb_cubemaps as usize * 6 * size.product() * self.internal_format.pixel_bits().expect("This internal format has no defined pixel size");
        (bits + 7) / 8
    }
}