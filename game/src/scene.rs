use std::collections::{HashMap, VecDeque};
use gx::gl::{self, types::GLenum};
use fate::vek::{Vec3, Rgba};

#[derive(Debug)]
pub struct Mesh {
    pub topology: GLenum,
    pub vposition: Vec<Vec3<f32>>, // Not optional
    pub vcolor: Vec<Rgba<f32>>, // Optional. If there's only one element, it is used for all vertices.
    pub indices: Vec<u16>, // Optional. If empty, it's rendered using glDrawArrays.
}

impl Mesh {
    fn new_cube_triangle_strip(s: f32) -> Self {
        let vposition = [
            Vec3::new(-s,  s,  s), // Front-top-left
            Vec3::new( s,  s,  s), // Front-top-right
            Vec3::new(-s, -s,  s), // Front-bottom-left
            Vec3::new( s, -s,  s), // Front-bottom-right
            Vec3::new( s, -s, -s), // Back-bottom-right
            Vec3::new( s,  s,  s), // Front-top-right
            Vec3::new( s,  s, -s), // Back-top-right
            Vec3::new(-s,  s,  s), // Front-top-left
            Vec3::new(-s,  s, -s), // Back-top-left
            Vec3::new(-s, -s,  s), // Front-bottom-left
            Vec3::new(-s, -s, -s), // Back-bottom-left
            Vec3::new( s, -s, -s), // Back-bottom-right
            Vec3::new(-s,  s, -s), // Back-top-left
            Vec3::new( s,  s, -s), // Back-top-right
        ];

        Self {
            topology: gl::TRIANGLE_STRIP,
            vposition: vposition.to_vec(),
            vcolor: vec![Rgba::red()],
            indices: vec![],
        }
    }
    pub fn new_cube() -> Self {
        Self::new_cube_triangle_strip(0.5)
    }
}

pub type MeshID = u32;

#[derive(Debug)]
pub enum SceneCommand {
    MeshUpdated { mesh_id: MeshID }
}

#[derive(Debug)]
pub struct Scene {
    pub meshes: HashMap<MeshID, Mesh>,
    pub command_queue: VecDeque<SceneCommand>,
}

impl Scene {
    pub fn new() -> Self {
        let cube_mesh_id = 1;
        let mut meshes = HashMap::new();
        let mut command_queue = VecDeque::new();

        meshes.insert(cube_mesh_id, Mesh::new_cube());
        command_queue.push_back(SceneCommand::MeshUpdated { mesh_id: cube_mesh_id });

        Self {
            meshes,
            command_queue,
        }
    }
}

