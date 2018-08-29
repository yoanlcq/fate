
#[derive(Debug, Clone, PartialEq)]
pub struct MeshInstance {
    pub mesh_id: MeshID,
    pub xform: Transform<f32, f32, f32>, // TODO: In the future, Xform is a component
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

    // Later we may also want a tick_commands_queue
    pub draw_commands_queue: VecDeque<SceneCommand>,
}

impl Scene {
    pub const MESHID_SKYBOX: MeshID = 10;
    pub const MESHID_CUBE: MeshID = 11;
    pub const MESHID_CUBE_SMOOTH: MeshID = 12;
    pub const MESHID_ICOSAHEDRON_0: MeshID = 13;
    pub const MESHID_ICOSAHEDRON_1: MeshID = 14;
    pub const MESHID_ICOSAHEDRON_2: MeshID = 15;
    pub const MESHID_ICOSAHEDRON_3: MeshID = 16;

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
        let gui_camera = Camera {
            position: Vec3::new(0., 0., -0.001),
            target: Vec3::new(0., 0., 1.),
            scale: Vec3::one(),
            viewport_size,
            projection_mode: CameraProjectionMode::Ortho,
            fov_y_radians: 60_f32.to_radians(),
            near: 0.001,
            far: 1.,
        };

        meshes.insert(Self::MESHID_SKYBOX, Mesh::new_skybox());
        meshes.insert(Self::MESHID_CUBE, Mesh::new_cube_triangles(0.5));
        meshes.insert(Self::MESHID_CUBE_SMOOTH, Mesh::new_cube_smooth_triangle_strip(0.5));
        meshes.insert(Self::MESHID_ICOSAHEDRON_0, Mesh::new_icosahedron(0.5, 0));
        meshes.insert(Self::MESHID_ICOSAHEDRON_1, Mesh::new_icosahedron(0.5, 1));
        meshes.insert(Self::MESHID_ICOSAHEDRON_2, Mesh::new_icosahedron(0.5, 2));
        meshes.insert(Self::MESHID_ICOSAHEDRON_3, Mesh::new_icosahedron(0.5, 3));
        draw_commands_queue.push_back(SceneCommand::AddMesh(Self::MESHID_SKYBOX));
        draw_commands_queue.push_back(SceneCommand::AddMesh(Self::MESHID_CUBE));
        draw_commands_queue.push_back(SceneCommand::AddMesh(Self::MESHID_CUBE_SMOOTH));
        draw_commands_queue.push_back(SceneCommand::AddMesh(Self::MESHID_ICOSAHEDRON_0));
        draw_commands_queue.push_back(SceneCommand::AddMesh(Self::MESHID_ICOSAHEDRON_1));
        draw_commands_queue.push_back(SceneCommand::AddMesh(Self::MESHID_ICOSAHEDRON_2));
        draw_commands_queue.push_back(SceneCommand::AddMesh(Self::MESHID_ICOSAHEDRON_3));

        let cube0_instance_id = 0;
        let cube1_instance_id = 1;
        let icosahedron0_instance_id = 2;
        let icosahedron1_instance_id = 3;

        mesh_instances.insert(cube0_instance_id, MeshInstance { mesh_id: Self::MESHID_CUBE, xform: Default::default() });
        mesh_instances.insert(cube1_instance_id, MeshInstance { mesh_id: Self::MESHID_CUBE_SMOOTH,
            xform: Transform {
                position: Vec3::new(-2., 0., 0.),
                .. Default::default()
            },
        });
        mesh_instances.insert(icosahedron0_instance_id, MeshInstance {
            mesh_id: Self::MESHID_ICOSAHEDRON_2,
            xform: Transform {
                position: Vec3::new(2., 0., 0.),
                .. Default::default()
            },
        });
        mesh_instances.insert(icosahedron1_instance_id, MeshInstance {
            mesh_id: Self::MESHID_ICOSAHEDRON_3,
            xform: Transform {
                position: Vec3::new(0., 2., 0.),
                .. Default::default()
            },
        });

        draw_commands_queue.push_back(SceneCommand::AddMeshInstance(cube0_instance_id));
        draw_commands_queue.push_back(SceneCommand::AddMeshInstance(cube1_instance_id));
        draw_commands_queue.push_back(SceneCommand::AddMeshInstance(icosahedron0_instance_id));
        draw_commands_queue.push_back(SceneCommand::AddMeshInstance(icosahedron1_instance_id));

        let skybox_selector = SkyboxSelector {
            tab: 0, layer: 0,
        };

        Self {
            skybox_min_mag_filter: gl::LINEAR,
            skybox_selector,
            cameras,
            gui_camera,
            meshes,
            mesh_instances,
            draw_commands_queue,
        }
    }
}


