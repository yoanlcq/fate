use fate::math::{Vec3, Rgba};

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mesh3DID(u32);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Mesh3DInfo {
    pub nb_vertices: usize,
    pub default_color: Rgba<u8>,
    // TODO: Spécifier l'usage de chaque buffer (static ? dynamic ? stream ?)
    // TODO: Spécifier un matériel par défaut
    pub keep_in_cpu: bool,
    pub v_position: Vec<Vec3<f32>>,
    pub v_normal: Vec<Vec3<f32>>,
    // pub v_tangent: Vec<Vec3<f32>>,
    // pub v_bitangent: Vec<Vec3<f32>>,
    pub v_color: Vec<Rgba<u8>>,
    // pub v_uv: Vec<Vec2<f32>>,
    pub indices: Vec<u32>,
}

// TODO: Pouvoir créer des instances ??

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Mesh3DChannel {
    Position = 0,
    Normal = 1,
    Tangent = 2,
    Bitangent = 3,
    Color = 4,
    UV = 5,
}