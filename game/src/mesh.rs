use fate::math::{Vec3, Rgba, Vec2, Mat4};
use texture2d::Texture2DSelector as Tex2D;
use fate::gx;

// TODO: Spécifier l'usage de chaque buffer (static ? dynamic ? stream ?) (glBufferStorage: updatable ou pas).
// Réponse: pas besoin. On n'autorise pas de resize, et on a besoin que le buffer soit
// updatable après sa création (on connaît pas tout d'un coup)

// Database of meshes (topology + indices + vertex data (non-instanced)).
// Database of instances (material type (shaderID) => mesh ID + instanced vertex data (model matrices, material index, etc)).
// Database of materials (uniform buffer of structs)

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
//
// Making the following assumptions :
// - Every mesh is a 3D mesh that has position, normal and UVs. We may want to support other kinds of data sets later (e.g for 2D sprites, and 2D vector art).
// - All meshes use indexed rendering, AND all indices are 32-bit (we may want to support non-indexed rendering later).
// - The "material index" indexes into a uniform array of PBR materials (implying all 3D meshes are PBR, which might not be true, especially 2D meshes).
// - Vertex data is kept in the CPU. We may want a GPU-only mode.
// - Buffers are not resizable (create with glBufferStorage());
// - Buffers can be updated after creation;

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MeshID(u32);

#[derive(Debug, Clone, PartialEq)]
pub struct MeshInfo {
    // --- Info that needs to be known at creation time ---
    pub nb_vertices: u32,
    pub nb_indices: u32,
    pub topology: gx::Topology,

    pub indices: Vec<u32>,
    pub v_position: Vec<Vec3<f32>>,
    pub v_normal: Vec<Vec3<f32>>,
    pub v_uv: Vec<Vec2<f32>>,
    pub i_model_matrix: Vec<Mat4<f32>>,
    pub i_material_index: Vec<u16>,
}

/// Vertex attrib indices
// NOTE: OpenGL mandates a minimum of 16.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum VertexAttribIndex {
    // Non-instanced
    Position = 0,
    Normal = 1,
    Tangent = 2,
    Bitangent = 3,
    Color = 4,
    UV = 5,
    UV2 = 6,
    UV3 = 7,
    UV4 = 8,
    // Instanced
    ModelMatrix = 9,
    MaterialIndex = 13,
}
