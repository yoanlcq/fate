use fate::math::{Extent2};
use gpu::GpuTextureInternalFormat;

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Texture2DSelector {
    pub array_id: Texture2DArrayID,
    pub slot: u16,
}

assert_eq_size!(tex2d_size; Texture2DSelector, u32);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Texture2DArrayID(pub u8);

impl Texture2DArrayID {
    pub const MAX: usize = 16;
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Texture2DArrayInfo {
    // Parameters at creation
    pub nb_levels: u32,
    pub internal_format: GpuTextureInternalFormat,
    pub size: Extent2<u32>,
    pub nb_slots: u32,
}

impl Texture2DArrayInfo {
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
        let bits = self.nb_slots as usize * size.product() * self.internal_format.pixel_bits().expect("This internal format has no defined pixel size");
        (bits + 7) / 8
    }
}