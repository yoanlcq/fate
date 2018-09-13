use fate::math::Rgba;
use cubemap::{CubemapArrayID, CubemapFace};
use texture2d::Texture2DArrayID;
use super::{CpuSubImage2D, GpuTextureFilter};

/// Commands for the rendering backend to update the on-GPU data.
///
/// They essentially indicate "what has changed" with only enough information for the renderer to
/// know where to find relevant data in the shared state.
#[derive(Debug, Clone, PartialEq)]
pub enum GpuCmd {
    ClearColorEdit,
    CubemapArrayCreate(CubemapArrayID),
    CubemapArrayDelete(CubemapArrayID),
    CubemapArrayClear(CubemapArrayID, u32, Rgba<f32>), // id, level, color
    CubemapArraySubImage2D(CubemapArrayID, usize, CubemapFace, CpuSubImage2D),
    CubemapArraySetMinFilter(CubemapArrayID, GpuTextureFilter),
    CubemapArraySetMagFilter(CubemapArrayID, GpuTextureFilter),
    Texture2DArrayCreate(Texture2DArrayID),
    Texture2DArrayDelete(Texture2DArrayID),
    Texture2DArrayClear(Texture2DArrayID, u32, Rgba<f32>), // id, level, color
    Texture2DArraySubImage2D(Texture2DArrayID, usize, CpuSubImage2D),
    Texture2DArraySetMinFilter(Texture2DArrayID, GpuTextureFilter),
    Texture2DArraySetMagFilter(Texture2DArrayID, GpuTextureFilter),
}

