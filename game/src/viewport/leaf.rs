use fate::math::Rgba;
use cubemap::CubemapSelector;
use eid::EID;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LeafViewport {
    // TODO: Describes what a viewport displays    
    pub clear_color: Rgba<f32>,
    pub skybox_cubemap_selector: Option<CubemapSelector>, // If None, skybox is disabled
    pub camera: EID, // TODO: Multiple (stacked) cameras (but draw skybox once with one of them)
}

