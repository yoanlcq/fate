use fate::math::Rgba;
use texture2d::Texture2DSelector as Tex2D;

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MaterialID(pub u32);

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Material {
    pub albedo_mul: Rgba<u8>,
    pub albedo_map: Tex2D,
    pub normal_map: Tex2D,
    pub metallic_mul: f32,
    pub metallic_map: Tex2D,
    pub roughness_mul: f32,
    pub roughness_map: Tex2D,
    pub ao_map: Tex2D,
}

