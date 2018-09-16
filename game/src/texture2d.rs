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
    pub const MAX: usize = 32;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Texture2DArrayInfo {
    // Parameters at creation
    pub nb_levels: u32,
    pub internal_format: GpuTextureInternalFormat,
    pub size: Extent2<u32>,
    pub nb_slots: u32,
}
