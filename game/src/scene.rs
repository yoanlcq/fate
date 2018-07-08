use std::collections::{HashMap, VecDeque};
use gx::gl::{self, types::GLenum};
use fate::vek::{Vec3, Vec4, Rgba, Transform};
use system::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub topology: GLenum,
    pub vposition: Vec<Vec4<f32>>, // Not optional
    pub vnormal: Vec<Vec4<f32>>, // Not optional
    pub vcolor: Vec<Rgba<u8>>, // Optional. If there's only one element, it is used for all vertices.
    pub indices: Vec<u16>, // Optional. If empty, it's rendered using glDrawArrays.
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeshInstance {
    pub mesh_id: MeshID,
    pub xform: Transform<f32, f32, f32>, // TODO: In the future, Xform is a component
}

impl Mesh {
    // A skybox is special because face winding is inverted so that we don't need to change cull face state.
    pub fn new_skybox() -> Self {
        let mut m = Self::new_cube_smooth_triangle_strip(0.5);

        // Flip winding by inserting a degenerate triangle
        let pos = m.vposition[0];
        let norm = m.vnormal[0];
        m.vposition.insert(0, pos);
        m.vnormal.insert(0, norm);

        // ... and reverse normals too (not that they are expected to be used anyway...)
        for n in &mut m.vnormal {
            *n = -*n;
        }

        // Make sure to make it opaque white
        for col in &mut m.vcolor {
            *col = Rgba::white();
        }

        m
    }
    pub fn new_cube_smooth_triangle_strip(s: f32) -> Self {
        let vposition = [
            Vec4::new(-s,  s,  s, 1.), // Front-top-left
            Vec4::new( s,  s,  s, 1.), // Front-top-right
            Vec4::new(-s, -s,  s, 1.), // Front-bottom-left
            Vec4::new( s, -s,  s, 1.), // Front-bottom-right
            Vec4::new( s, -s, -s, 1.), // Back-bottom-right
            Vec4::new( s,  s,  s, 1.), // Front-top-right
            Vec4::new( s,  s, -s, 1.), // Back-top-right
            Vec4::new(-s,  s,  s, 1.), // Front-top-left
            Vec4::new(-s,  s, -s, 1.), // Back-top-left
            Vec4::new(-s, -s,  s, 1.), // Front-bottom-left
            Vec4::new(-s, -s, -s, 1.), // Back-bottom-left
            Vec4::new( s, -s, -s, 1.), // Back-bottom-right
            Vec4::new(-s,  s, -s, 1.), // Back-top-left
            Vec4::new( s,  s, -s, 1.), // Back-top-right
        ];

        Self {
            topology: gl::TRIANGLE_STRIP,
            vposition: vposition.to_vec(),
            vnormal: vposition.iter().cloned().map(|mut p| { p.w = 0.; p.normalize(); p.w = 0.; p }).collect(),
            vcolor: vec![Rgba::red()],
            indices: vec![],
        }
    }
    pub fn new_cube_triangles(s: f32) -> Self {
        let v = (
            Vec4::new(-s,  s, -s, 1.), // 0
            Vec4::new( s,  s, -s, 1.), // 1
            Vec4::new( s,  s,  s, 1.), // 2
            Vec4::new(-s,  s,  s, 1.), // 3
            Vec4::new(-s, -s,  s, 1.), // 4
            Vec4::new(-s, -s, -s, 1.), // 5
            Vec4::new( s, -s, -s, 1.), // 6
            Vec4::new( s, -s,  s, 1.), // 7
        );
        let vposition = [
            v.7, v.2, v.1,
            v.7, v.1, v.6,
            v.4, v.5, v.0,
            v.4, v.0, v.3,
            v.0, v.1, v.2,
            v.0, v.2, v.3,
            v.5, v.4, v.7,
            v.5, v.7, v.6,
            v.4, v.3, v.2,
            v.4, v.2, v.7,
            v.1, v.0, v.5,
            v.1, v.5, v.6,
        ];
        let vnormal = [
            Vec4::right(),
            Vec4::right(),
            Vec4::right(),
            Vec4::right(),
            Vec4::right(),
            Vec4::right(),
            Vec4::left(),
            Vec4::left(),
            Vec4::left(),
            Vec4::left(),
            Vec4::left(),
            Vec4::left(),
            Vec4::up(),
            Vec4::up(),
            Vec4::up(),
            Vec4::up(),
            Vec4::up(),
            Vec4::up(),
            Vec4::down(),
            Vec4::down(),
            Vec4::down(),
            Vec4::down(),
            Vec4::down(),
            Vec4::down(),
            Vec4::forward_lh(),
            Vec4::forward_lh(),
            Vec4::forward_lh(),
            Vec4::forward_lh(),
            Vec4::forward_lh(),
            Vec4::forward_lh(),
            Vec4::back_lh(),
            Vec4::back_lh(),
            Vec4::back_lh(),
            Vec4::back_lh(),
            Vec4::back_lh(),
            Vec4::back_lh(),
        ];
        Self {
            topology: gl::TRIANGLES,
            vposition: vposition.to_vec(),
            vnormal: vnormal.to_vec(),
            vcolor: vec![Rgba::green()],
            indices: vec![],
        }
    }
    pub fn new_cube() -> Self {
        Self::new_cube_triangles(0.5)
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum CameraProjectionMode {
    Perspective,
    Ortho,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Camera {
    pub position: Vec3<f32>,
    pub target: Vec3<f32>,
    pub scale: Vec3<f32>,
    pub projection_mode: CameraProjectionMode,
    pub fov_y_radians: f32,
    pub near: f32,
    pub far: f32,
}

pub type MeshID = u32;
pub type MeshInstanceID = u32;
pub type CameraID = u32;

#[derive(Debug)]
pub enum SceneCommand {
    AddMesh(MeshID),
    AddMeshInstance(MeshInstanceID),
}

#[derive(Debug)]
pub struct Scene {
    pub cameras: HashMap<CameraID, Camera>,
    pub meshes: HashMap<MeshID, Mesh>,
    pub mesh_instances: HashMap<MeshInstanceID, MeshInstance>,
    // Later we may also want a tick_commands_queue
    pub draw_commands_queue: VecDeque<SceneCommand>,
}

impl Scene {
    pub const MESHID_SKYBOX: MeshID = 10;
    pub const MESHID_CUBE: MeshID = 11;
    pub const MESHID_CUBE_SMOOTH: MeshID = 12;

    pub fn new() -> Self {
        let mut cameras = HashMap::new();
        let mut meshes = HashMap::new();
        let mut mesh_instances = HashMap::new();
        let mut draw_commands_queue = VecDeque::new();

        cameras.insert(1, Camera {
            position: Vec3::new(0., 0., -5.),
            target: Vec3::zero(),
            scale: Vec3::one(),
            projection_mode: CameraProjectionMode::Perspective,
            fov_y_radians: 60_f32.to_radians(),
            near: 0.001,
            far: 10000.,
        });

        meshes.insert(Self::MESHID_SKYBOX, Mesh::new_skybox());
        meshes.insert(Self::MESHID_CUBE, Mesh::new_cube_triangles(0.5));
        meshes.insert(Self::MESHID_CUBE_SMOOTH, Mesh::new_cube_smooth_triangle_strip(0.5));
        draw_commands_queue.push_back(SceneCommand::AddMesh(Self::MESHID_SKYBOX));
        draw_commands_queue.push_back(SceneCommand::AddMesh(Self::MESHID_CUBE));
        draw_commands_queue.push_back(SceneCommand::AddMesh(Self::MESHID_CUBE_SMOOTH));


        mesh_instances.insert(1300, MeshInstance {
            mesh_id: Self::MESHID_SKYBOX,
            xform: Transform {
                scale: Vec3::broadcast(1000.),
                .. Default::default()
            },
        });
        draw_commands_queue.push_back(SceneCommand::AddMeshInstance(1300));

        mesh_instances.insert(13, MeshInstance {
            mesh_id: Self::MESHID_CUBE,
            xform: Default::default(),
        });
        draw_commands_queue.push_back(SceneCommand::AddMeshInstance(13));
        mesh_instances.insert(42, MeshInstance {
            mesh_id: Self::MESHID_CUBE,
            xform: Transform {
                position: Vec3::new(2., 0., 0.),
                .. Default::default()
            },
        });
        draw_commands_queue.push_back(SceneCommand::AddMeshInstance(42));
        mesh_instances.insert(468, MeshInstance {
            mesh_id: Self::MESHID_CUBE_SMOOTH,
            xform: Transform {
                position: Vec3::new(-2., 0., 0.),
                .. Default::default()
            },
        });
        draw_commands_queue.push_back(SceneCommand::AddMeshInstance(468));


        Self {
            cameras,
            meshes,
            mesh_instances,
            draw_commands_queue,
        }
    }
}

// Add this system _after_ any renderer.
#[derive(Debug)]
pub struct SceneCommandClearerSystem;

impl SceneCommandClearerSystem {
    pub fn new() -> Self {
        SceneCommandClearerSystem
    }
}

impl System for SceneCommandClearerSystem {
    fn draw(&mut self, g: &mut G, _: &Draw) {
        g.scene.draw_commands_queue.clear();
    }
}

#[derive(Debug)]
pub struct SceneLogicSystem;

impl SceneLogicSystem {
    pub fn new() -> Self {
        SceneLogicSystem
    }
}

impl System for SceneLogicSystem {
    fn draw(&mut self, g: &mut G, draw: &Draw) {
        for i in g.scene.mesh_instances.values_mut() {
            i.xform.orientation.rotate_x(90_f32.to_radians() * draw.dt);
        }
    }
}

