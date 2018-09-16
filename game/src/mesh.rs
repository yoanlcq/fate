use fate::math::{Vec3, Rgba, Vec2, Mat4};
use texture2d::Texture2DSelector as Tex2D;

// TODO: Spécifier l'usage de chaque buffer (static ? dynamic ? stream ?) (glBufferStorage: updatable ou pas).
// Réponse: pas besoin. On n'autorise pas de resize, et on a besoin que le buffer soit
// updatable après sa création (on connaît pas tout d'un coup)

// Database of meshes (topology + indices + vertex data (non-instanced)).
// Database of instances (material type (shaderID) => mesh ID + instanced vertex data (model matrices, material index, etc)).
// Database of materials (uniform buffer of structs)

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MeshID(u32);

#[derive(Debug, Clone, PartialEq)]
pub struct MeshInfo {
    // --- Info that needs to be know at creation time ---
    pub nb_vertices: u32,
    pub nb_indices: u32, // 0 if no indices, non-zero otherwise.
    pub topology: Topology,

    pub indices: Buffer, // u8, u16, or u32
    pub vertex_data: BTreeMap<Channel, Buffer>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MeshInstances {
    pub arrays: BTreeMap<Channel, Buffer>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Buffer {
    ty: ....,
    data: Vec<u8>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
// NOTE: Up to 16 vec4 registers.
pub enum Channel {
    // Non-instanced attributes
    Position = 0,
    Normal = 1,
    Tangent = 2,
    Bitangent = 3,
    Color = 4,
    UV = 5,
    // Instanced vertex attributes
    ModelMatrix = 6,
    MaterialIndex = 10,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct PBRMaterial {
    pub albedo_mul   : Rgba<u8>,
    pub albedo_map   : Tex2D,
    pub normal_map   : Tex2D,
    pub metallic_mul : f32,
    pub metallic_map : Tex2D,
    pub roughness_mul: f32,
    pub roughness_map: Tex2D,
    pub ao_map       : Tex2D,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Topology {
    Points                 = gl::POINTS,
    LineStrip              = gl::LINE_STRIP,
    LineLoop               = gl::LINE_LOOP,
    Lines                  = gl::LINES,
    LineStripAdjacency     = gl::LINE_STRIP_ADJACENCY,
    LinesAdjacency         = gl::LINES_ADJACENCY,
    TriangleStrip          = gl::TRIANGLE_STRIP,
    TriangleFan            = gl::TRIANGLE_FAN,
    Triangles              = gl::TRIANGLES,
    TriangleStripAdjacency = gl::TRIANGLE_STRIP_ADJACENCY,
    TrianglesAdjacency     = gl::TRIANGLES_ADJACENCY,
    Patches                = gl::PATCHES,
}


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

