use cubemap::{CubemapArrayID, CubemapFace};

/// Commands for the rendering backend to update the on-GPU data.
///
/// They essentially indicate "what has changed" with only enough information for the renderer to
/// know where to find relevant data in the shared state.
#[derive(Debug, Clone, PartialEq)]
pub enum GpuCmd {
    ClearColorEdit,
    CubemapArrayCreate(CubemapArrayID),
    CubemapArrayDelete(CubemapArrayID),
    CubemapArrayClear(CubemapArrayID),
    CubemapArraySubImage(CubemapArrayID, usize, CubemapFace),
}


