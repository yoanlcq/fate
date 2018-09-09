use fate::math::{Vec3, Rgba, Vec2, Mat4};
use texture2d::Texture2DSelector as Tex2D;

// GL_POINTS, GL_LINE_STRIP, GL_LINE_LOOP, GL_LINES, GL_LINE_STRIP_ADJACENCY, GL_LINES_ADJACENCY, GL_TRIANGLE_STRIP, GL_TRIANGLE_FAN, GL_TRIANGLES
// GL_TRIANGLE_STRIP_ADJACENCY, GL_TRIANGLES_ADJACENCY, and GL_PATCHES

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MeshID(u32);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MeshInfo {
    // --- Info that needs to be know at creation time ---
    pub nb_vertices: u32,
    pub nb_indices: u32, // 0 if no indices, non-zero otherwise.
    // pub topology: Topology,
    // TODO: Spécifier l'usage de chaque buffer (static ? dynamic ? stream ?) (glBufferStorage: updatable ou pas)
    // TODO: Spécifier quelles sources de données sont utilisées (en gros: C'est des components !)

    pub keep_in_cpu: bool,
}

// All of the structs below are components that a mesh opts into.
// A mesh can be either indxed or non-indexed

// Index components
pub struct Indices_U32        { pub data: Vec<u32>, }
pub struct Indices_U16        { pub data: Vec<u16>, }
pub struct Indices_U8         { pub data: Vec<u8>, }
// Non-instanced attributes (2D meshes)
pub struct V_Positions_2_F32  { pub data: Vec<Vec2<f32>>, }
// Non-instanced attributes (3D meshes)
pub struct V_Positions_3_F32  { pub data: Vec<Vec3<f32>>, }
pub struct V_Normals_3_F32    { pub data: Vec<Vec3<f32>>, }
pub struct V_Tangents_3_F32   { pub data: Vec<Vec3<f32>>, }
pub struct V_Bitangents_3_F32 { pub data: Vec<Vec3<f32>>, }
// Non-instanced attributes (any mesh)
pub struct V_Color_4_U8       { pub data: Vec<Rgba<u8>>, }
pub struct V_Uv_2_F32         { pub data: Vec<Vec2<f32>>, }
// Instanced attributes. (Material stuff may be reduced to material IDs later)
pub struct I_ModelMatrix      { pub data: Vec<Mat4<f32>>, }
pub struct I_AlbedoMul        { pub data: Vec<f32>, }
pub struct I_AlbedoMap        { pub data: Vec<Tex2D>,  }
pub struct I_NormalMul        { pub data: Vec<f32>, }
pub struct I_NormalMap        { pub data: Vec<Tex2D>, }
pub struct I_MetallicMul      { pub data: Vec<f32>, }
pub struct I_MetallicMap      { pub data: Vec<Tex2D>, }
pub struct I_RoughnessMul     { pub data: Vec<f32>, }
pub struct I_RoughnessMap     { pub data: Vec<Tex2D>, }
pub struct I_AoMul            { pub data: Vec<f32>, }
pub struct I_AoMap            { pub data: Vec<Tex2D>, }

// Rendering the window:
// - Opaque 2D pass
// - Opaque 3D pass
// - Skybox pass
// - Transparent 3D pass
// - Transparent 2D pass
//
// In a "pass", the following happens:
// - For each GL program (2D, PBR, Phong, Skybox)
//   - For each viewport
//     - For each viewport-layer (camera + visual layers) of the viewport
//       - glMultiDraw*Indirect([vertex_attributes, instanced_attributes])
//

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum MeshChannel {
    Position = 0,
    Normal = 1,
    Tangent = 2,
    Bitangent = 3,
    Color = 4,
    UV = 5,
}
