use std::collections::{HashMap, VecDeque};
use fate::math::{Vec3, Vec4, Rgba, Transform, Quaternion, Mat4, FrustumPlanes};
use fate::gx::gl::{self, types::GLenum};
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
    pub fn new_icosahedron(s: f32, nb_subdivisions: usize) -> Self {
        let t = (1. + 5_f32.sqrt()) / 2.;
        let mut vertices = vec![
            Vec3::new(-1.0,  t, 0.0).normalized() * s,
            Vec3::new( 1.0,  t, 0.0).normalized() * s,
            Vec3::new(-1.0, -t, 0.0).normalized() * s,
            Vec3::new( 1.0, -t, 0.0).normalized() * s,
            Vec3::new(0.0, -1.0,  t).normalized() * s,
            Vec3::new(0.0,  1.0,  t).normalized() * s,
            Vec3::new(0.0, -1.0, -t).normalized() * s,
            Vec3::new(0.0,  1.0, -t).normalized() * s,
            Vec3::new( t, 0.0, -1.0).normalized() * s,
            Vec3::new( t, 0.0,  1.0).normalized() * s,
            Vec3::new(-t, 0.0, -1.0).normalized() * s,
            Vec3::new(-t, 0.0,  1.0).normalized() * s,
        ];
        let mut indices = vec![
            0, 11, 5,
            0, 5, 1,
            0, 1, 7,
            0, 7, 10,
            0, 10, 11,
            1, 5, 9,
            5, 11, 4,
            11, 10, 2,
            10, 7, 6,
            7, 1, 8,
            3, 9, 4,
            3, 4, 2,
            3, 2, 6,
            3, 6, 8,
            3, 8, 9,
            4, 9, 5,
            2, 4, 11,
            6, 2, 10,
            8, 6, 7,
            9, 8, 1,
        ];

        for _ in 0..nb_subdivisions {
            let mut out_vertices = vec![];
            let mut out_indices = vec![];
            for face in indices.chunks(3) {
                let v0 = vertices[face[0] as usize];
                let v1 = vertices[face[1] as usize];
                let v2 = vertices[face[2] as usize];
                let v3 = ((v0 + v1) / 2.).normalized() * s;
                let v4 = ((v1 + v2) / 2.).normalized() * s;
                let v5 = ((v2 + v0) / 2.).normalized() * s;
                let i = out_vertices.len() as u16;
                out_vertices.extend(&[v0, v1, v2, v3, v4, v5]);
                out_indices.extend(&[i+0, i+3, i+5]);
                out_indices.extend(&[i+3, i+1, i+4]);
                out_indices.extend(&[i+5, i+4, i+2]);
                out_indices.extend(&[i+3, i+4, i+5]);
            }
            vertices = out_vertices;
            indices = out_indices;
        }

        Self {
            topology: gl::TRIANGLES,
            vposition: vertices.iter().cloned().map(Vec4::from_point).collect(),
            vnormal: vertices.iter().cloned().map(Vec4::from_direction).collect(),
            vcolor: vec![Rgba::blue()],
            indices,
        }
    }

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
    pub viewport_size: Extent2<u32>,
    pub projection_mode: CameraProjectionMode,
    pub fov_y_radians: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn forward(&self) -> Vec3<f32> {
        (self.target - self.position).normalized()
    }
    pub fn up(&self) -> Vec3<f32> {
        self.forward().cross(self.right())
    }
    pub fn right(&self) -> Vec3<f32> {
        self.up_vector_for_lookat().cross(self.forward())
    }
    // !!! Must be normalized
    pub fn up_vector_for_lookat(&self) -> Vec3<f32> {
        Vec3::up()
    }
    pub fn aspect_ratio(&self) -> f32 {
        let Extent2 { w, h } = self.viewport_size;
        assert_ne!(w, 0);
        assert_ne!(h, 0);
        w as f32 / h as f32
    }
    pub fn ortho_frustum_planes(&self) -> FrustumPlanes<f32> {
        let aspect_ratio = self.aspect_ratio();
        FrustumPlanes {
            right: aspect_ratio,
            left: -aspect_ratio,
            top: 1.,
            bottom: -1.,
            near: self.near,
            far: self.far,
        }
    }
    pub fn proj_matrix(&self) -> Mat4<f32> {
        match self.projection_mode {
            CameraProjectionMode::Perspective => {
                Mat4::perspective_lh_no(self.fov_y_radians, self.aspect_ratio(), self.near, self.far)
            },
            CameraProjectionMode::Ortho => {
                Mat4::orthographic_lh_no(self.ortho_frustum_planes())
            },
        }
    }
    pub fn view_matrix(&self) -> Mat4<f32> {
        let zoom = Mat4::<f32>::scaling_3d(self.scale.recip());
        let look = Mat4::look_at(self.position, self.target, Vec3::up());
        zoom * look
    }
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
    pub skybox_selector: SkyboxSelector,
    pub skybox_min_mag_filter: GLenum,
    pub meshes: HashMap<MeshID, Mesh>,
    pub mesh_instances: HashMap<MeshInstanceID, MeshInstance>,
    // Later we may also want a tick_commands_queue
    pub draw_commands_queue: VecDeque<SceneCommand>,
}

impl Scene {
    pub const MESHID_SKYBOX: MeshID = 10;
    pub const MESHID_CUBE: MeshID = 11;
    pub const MESHID_CUBE_SMOOTH: MeshID = 12;

    pub fn new(viewport_size: Extent2<u32>) -> Self {
        let mut cameras = HashMap::new();
        let mut meshes = HashMap::new();
        let mut mesh_instances = HashMap::new();
        let mut draw_commands_queue = VecDeque::new();

        cameras.insert(1, Camera {
            position: Vec3::new(0., 0., -5.),
            target: Vec3::zero(),
            scale: Vec3::one(),
            viewport_size,
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

        let skybox_selector = SkyboxSelector {
            tab: 0, layer: 0,
        };

        Self {
            skybox_min_mag_filter: gl::LINEAR,
            skybox_selector,
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
    fn on_canvas_resized(&mut self, g: &mut G, size: Extent2<u32>) {
        for camera in g.scene.cameras.values_mut() {
            camera.viewport_size = size;
        }
    }
    fn on_key(&mut self, g: &mut G, key: Key, state: KeyState) {
        match key.sym {
            Some(Keysym::T) if state.is_down() && g.scene.skybox_selector.tab > 0 => g.scene.skybox_selector.tab -= 1,
            Some(Keysym::Y) if state.is_down() => g.scene.skybox_selector.tab += 1,
            Some(Keysym::U) if state.is_down() && g.scene.skybox_selector.layer > 0 => g.scene.skybox_selector.layer -= 1,
            Some(Keysym::I) if state.is_down() => g.scene.skybox_selector.layer += 1,
            Some(Keysym::O) if state.is_down() => g.scene.skybox_min_mag_filter = match g.scene.skybox_min_mag_filter {
                gl::LINEAR => gl::NEAREST,
                gl::NEAREST => gl::LINEAR,
                _ => gl::LINEAR,
            },
            _ => (),
        }
    }
    fn draw(&mut self, g: &mut G, draw: &Draw) {
        for i in g.scene.mesh_instances.values_mut() {
            i.xform.orientation.rotate_x(90_f32.to_radians() * draw.dt);
        }
        for camera in g.scene.cameras.values_mut() {
            // Translate
            let input = g.input.debug_camera_keyboard_dpad();
            let is_freefly = true; // Otherwise, it is "look at target"
            if input != Vec3::zero() { // Testing inequality is fine because it's a D-pad
                let camera_speed = 10.;
                let tx = camera.right() * input.x;
                let ty = camera.up() * input.y;
                let tz = camera.forward() * input.z;
                let t = (tx + ty + tz) * camera_speed * draw.dt;
                if is_freefly {
                    camera.position += t;
                    camera.target += t;
                }
            }

            // Rotate
            let disp = g.input.mouse_displacement();
            if g.input.mouse_button(MouseButton::Left).is_down() && disp != Vec2::zero() {
                let degrees_per_pixel = 0.6;
                let disp = disp.map(|x| (x * degrees_per_pixel).to_radians() as f32);
                let mut self_to_target = camera.target - camera.position;
                let rx = Quaternion::rotation_3d(disp.y, camera.right());
                let ry = Quaternion::rotation_3d(disp.x, camera.up());
                self_to_target = rx * ry * self_to_target;
                if is_freefly {
                    camera.target = camera.position + self_to_target;
                } else {
                    camera.position = camera.target - self_to_target;
                }
            }
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct TextureSelector {
    pub tab: u16,
    pub layer: u16,
}

pub type SkyboxSelector = TextureSelector;
