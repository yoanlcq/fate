use std::time::Duration;
use std::collections::{VecDeque, HashMap};
use std::sync::Arc;

use fate::mt;
use fate::math::{Extent2, Rgba, Rect};
use fate::math::{Vec2, Vec3, Mat4};
use fate::lab::fps::FpsStats;

use frame_time::FrameTimeManager;
use message::Message;
use input::Input;
use resources::Resources;
use gpu::{GpuCmd, CpuSubImage2D, GpuTextureFilter};
use mouse_cursor::MouseCursor;
use viewport::{ViewportDB, ViewportVisitor, ViewportInfo};
use cubemap::{CubemapArrayInfo, CubemapArrayID, CubemapFace, CubemapSelector};
use texture2d::{Texture2DArrayInfo, Texture2DArrayID};
use mesh::{MeshID, MeshInfo};
use material::{MaterialID, Material};
use light::Light;
use camera::{Camera, CameraProjectionMode};
use xform::Xform;
use eid::EID;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct MeshInstance {
    pub mesh_id: MeshID,
    pub material_id: MaterialID,
}

#[derive(Debug)]
pub struct G {
    /// Total physics time since the game started (accumulation of per-tick delta times)
    pub t: Duration, 

    pub frame_time_manager: FrameTimeManager,
    fps_stats_history: VecDeque<FpsStats>,

    pub mt: Arc<mt::SharedThreadContext>,

    pub res: Resources,

    pub pending_messages: VecDeque<Message>,
    pub input: Input,

    //
    // Main "world"
    //

    gpu_cmd_queue: VecDeque<GpuCmd>,

    // "singletons"
    pub is_mouse_cursor_visible: bool,
    pub mouse_cursor: MouseCursor,
    clear_color: Rgba<f32>,
    viewport_db: ViewportDB,

    /*
    skybox_is_enabled: bool,
    skybox_cubemap_selector: CubemapSelector,
    */

    //
    cubemap_arrays: [Option<CubemapArrayInfo>; CubemapArrayID::MAX],
    texture2d_arrays: [Option<Texture2DArrayInfo>; Texture2DArrayID::MAX],
    //meshes: HashMap<MeshID, MeshInfo>,
    //materials: HashMap<MaterialID, Material>,

    // "entities"
    xforms: HashMap<EID, Xform>,
    cameras: HashMap<EID, Camera>,
    //lights: HashMap<EID, Light>,
    //instances: HashMap<EID, MeshInstance>,

    /*
    visual_layers: HashMap<EID, VisualLayerID>,
    physics_layers: HashMap<EID, PhysicsLayerID>,
    visual_space: HashMap<EID, VisualSpace>,
    */
}

impl G {
    pub fn new(canvas_size: Extent2<u32>, mt: Arc<mt::SharedThreadContext>) -> Self {
        let camera = EID(0);
        let viewport_info = ViewportInfo {
            clear_color: Rgba::blue(),
            skybox_cubemap_selector: CubemapSelector { array_id: CubemapArrayID(0), cubemap: 1, },
            camera,
        };

        let mut g = Self {
            t: Duration::default(),
            frame_time_manager: FrameTimeManager::with_max_len(60),
            pending_messages: VecDeque::new(),
            fps_stats_history: VecDeque::new(),
            mt,
            input: Input::new(canvas_size),
            res: Resources::new().unwrap(),
            gpu_cmd_queue: VecDeque::with_capacity(1024),
            clear_color: Rgba::new(0., 1., 1., 1.),
            mouse_cursor: MouseCursor::default(),
            is_mouse_cursor_visible: true,
            viewport_db: ViewportDB::new(viewport_info),
            cubemap_arrays: array![None; CubemapArrayID::MAX],
            texture2d_arrays: array![None; Texture2DArrayID::MAX],
            //meshes: HashMap::new(),
            //materials: HashMap::new(),
            xforms: HashMap::new(),
            cameras: HashMap::new(),
            //lights: HashMap::new(),
            //instances: HashMap::new(),
        };
        g.gpu_cmd_queue.push_back(GpuCmd::ClearColorEdit);
        g.eid_set_xform(camera, Xform {
            position: Vec3::new(0., 0., -5.),
            .. Default::default()
        });
        g.eid_set_camera(camera, Camera {
            projection_mode: CameraProjectionMode::Perspective,
            fov_y_radians: 60_f32.to_radians(),
            near: 0.001,
            far: 10000.,
        });
        g
    }
    #[allow(dead_code)]
    pub fn push_message(&mut self, msg: Message) {
        self.pending_messages.push_back(msg);
    }
    pub fn push_fps_stats(&mut self, fps_stats: FpsStats) {
        // Pretend we only keep 1 entry in the history
        self.fps_stats_history.pop_front();
        self.fps_stats_history.push_back(fps_stats);
    }
    pub fn last_fps_stats(&self) -> Option<FpsStats> {
        self.fps_stats_history.back().map(Clone::clone)
    }
    pub fn gpu_cmd_queue(&self) -> &VecDeque<GpuCmd> {
        &self.gpu_cmd_queue
    }
    pub fn gpu_cmd_queue_clear(&mut self) {
        self.gpu_cmd_queue.clear()
    }
    pub fn clear_color(&self) -> Rgba<f32> {
        self.clear_color
    }

    pub fn eid_xform(&self, eid: EID) -> Option<&Xform> {
        self.xforms.get(&eid)
    }
    pub fn eid_xform_mut(&mut self, eid: EID) -> Option<&mut Xform> {
        self.xforms.get_mut(&eid)
    }
    pub fn eid_set_xform(&mut self, eid: EID, xform: Xform) -> Option<Xform> {
        self.xforms.insert(eid, xform)
    }
    pub fn eid_unset_xform(&mut self, eid: EID) -> Option<Xform> {
        self.xforms.remove(&eid)
    }

    pub fn eid_camera(&self, eid: EID) -> Option<&Camera> {
        self.cameras.get(&eid)
    }
    pub fn eid_camera_mut(&mut self, eid: EID) -> Option<&mut Camera> {
        self.cameras.get_mut(&eid)
    }
    pub fn eid_set_camera(&mut self, eid: EID, camera: Camera) -> Option<Camera> {
        self.cameras.insert(eid, camera)
    }
    pub fn eid_unset_camera(&mut self, eid: EID) -> Option<Camera> {
        self.cameras.remove(&eid)
    }
 
    pub fn viewport_db(&self) -> &ViewportDB {
        &self.viewport_db
    }
    pub fn viewport_db_mut(&mut self) -> &mut ViewportDB {
        &mut self.viewport_db
    }
    pub fn visit_viewports(&self, f: &mut ViewportVisitor) {
        let Extent2 { w, h } = self.input.canvas_size();
        self.viewport_db().visit(Rect { x: 0, y: 0, w, h }, f);
    }

    pub fn cubemap_array_create(&mut self, id: CubemapArrayID, info: CubemapArrayInfo) {
        assert!(self.cubemap_array_info(id).is_none());
        self.cubemap_arrays[id.0 as usize] = Some(info);
        self.gpu_cmd_queue.push_back(GpuCmd::CubemapArrayCreate(id))
    }
    pub fn cubemap_array_delete(&mut self, id: CubemapArrayID) -> Option<CubemapArrayInfo> {
        assert!(self.cubemap_array_info(id).is_some());
        self.gpu_cmd_queue.push_back(GpuCmd::CubemapArrayDelete(id));
        self.cubemap_arrays[id.0 as usize].take()
    }
    pub fn cubemap_array_info(&self, array: CubemapArrayID) -> Option<&CubemapArrayInfo> {
        self.cubemap_arrays[array.0 as usize].as_ref()
    }
    pub fn cubemap_array_clear(&mut self, array: CubemapArrayID, level: u32, color: Rgba<f32>) {
        assert!(self.cubemap_array_info(array).is_some());
        assert!(level < self.cubemap_array_info(array).unwrap().nb_levels);
        self.gpu_cmd_queue.push_back(GpuCmd::CubemapArrayClear(array, level, color))
    }
    pub fn cubemap_array_sub_image_2d(&mut self, array: CubemapArrayID, cubemap: usize, face: CubemapFace, img: CpuSubImage2D) {
        assert!(self.cubemap_array_info(array).is_some());
        assert!(cubemap < self.cubemap_array_info(array).unwrap().nb_cubemaps as usize);
        self.gpu_cmd_queue.push_back(GpuCmd::CubemapArraySubImage2D(array, cubemap, face, img))
    }
    pub fn cubemap_array_set_min_filter(&mut self, id: CubemapArrayID, filter: GpuTextureFilter) {
        assert!(self.cubemap_array_info(id).is_some());
        self.gpu_cmd_queue.push_back(GpuCmd::CubemapArraySetMinFilter(id, filter))
    }
    pub fn cubemap_array_set_mag_filter(&mut self, id: CubemapArrayID, filter: GpuTextureFilter) {
        assert!(self.cubemap_array_info(id).is_some());
        self.gpu_cmd_queue.push_back(GpuCmd::CubemapArraySetMagFilter(id, filter))
    }

    pub fn texture2d_array_create(&mut self, id: Texture2DArrayID, info: Texture2DArrayInfo) {
        assert!(self.texture2d_array_info(id).is_none());
        self.texture2d_arrays[id.0 as usize] = Some(info);
        self.gpu_cmd_queue.push_back(GpuCmd::Texture2DArrayCreate(id))
    }
    pub fn texture2d_array_delete(&mut self, id: Texture2DArrayID) -> Option<Texture2DArrayInfo> {
        assert!(self.texture2d_array_info(id).is_some());
        self.gpu_cmd_queue.push_back(GpuCmd::Texture2DArrayDelete(id));
        self.texture2d_arrays[id.0 as usize].take()
    }
    pub fn texture2d_array_info(&self, array: Texture2DArrayID) -> Option<&Texture2DArrayInfo> {
        self.texture2d_arrays[array.0 as usize].as_ref()
    }
    pub fn texture2d_array_clear(&mut self, array: Texture2DArrayID, level: u32, color: Rgba<f32>) {
        assert!(self.texture2d_array_info(array).is_some());
        assert!(level < self.texture2d_array_info(array).unwrap().nb_levels);
        self.gpu_cmd_queue.push_back(GpuCmd::Texture2DArrayClear(array, level, color))
    }
    pub fn texture2d_array_sub_image_2d(&mut self, array: Texture2DArrayID, slot: usize, img: CpuSubImage2D) {
        assert!(self.texture2d_array_info(array).is_some());
        assert!(slot < self.texture2d_array_info(array).unwrap().nb_slots as usize);
        self.gpu_cmd_queue.push_back(GpuCmd::Texture2DArraySubImage2D(array, slot, img))
    }
    pub fn texture2d_array_set_min_filter(&mut self, id: Texture2DArrayID, filter: GpuTextureFilter) {
        assert!(self.texture2d_array_info(id).is_some());
        self.gpu_cmd_queue.push_back(GpuCmd::Texture2DArraySetMinFilter(id, filter))
    }
    pub fn texture2d_array_set_mag_filter(&mut self, id: Texture2DArrayID, filter: GpuTextureFilter) {
        assert!(self.texture2d_array_info(id).is_some());
        self.gpu_cmd_queue.push_back(GpuCmd::Texture2DArraySetMagFilter(id, filter))
    }


    /*
    pub fn mesh_create(&mut self, info: MeshInfo) -> MeshID {
        // Push a command to ask "alloc nb_vertices and nb_indices" as specified in the info.
        unimplemented!()
    }
    pub fn mesh_delete(&mut self, mesh: MeshID) {
        // Push a command to free space occupied by this mesh
        unimplemented!()
    }
    pub fn mesh_info(&self, mesh: MeshID) -> Option<&MeshInfo> {
        // self.meshes.get(&mesh)
        unimplemented!()
    }
    pub fn mesh_set_indices(&mut self, mesh: MeshID, start: usize, data: Box<[u32]>) {
        // Push a command to call BufferSubData()
    }
    pub fn mesh_set_positions(&mut self, mesh: MeshID, start: usize, data: Box<[Vec3<f32>]>) {
        // Push a command to call BufferSubData()
    }
    pub fn mesh_set_normals(&mut self, mesh: MeshID, start: usize, data: Box<[Vec3<f32>]>) {
        // Push a command to call BufferSubData()
    }
    pub fn mesh_set_uvs(&mut self, mesh: MeshID, start: usize, data: Box<[Vec2<f32>]>) {
        // Push a command to call BufferSubData()
    }
*/
/*
    pub fn instance_array_create(&mut self, info: InstanceArrayInfo) -> InstanceArrayID {
        // Push a command to ask "alloc nb_instances" as specified in the info.
        unimplemented!()
    }
    pub fn instance_array_delete(&mut self, i: InstanceArrayID) {
        // Push a command to free the space occupied by this instance array
    }
    pub fn instance_array_info(&self, i: InstanceArrayID) -> Option<&InstanceArrayInfo> {
        self.instance_arrays.get(&i.0)
    }
    pub fn instance_array_set_model_matrices(&mut self, i: InstanceArrayID, start: usize, data: Box<[Mat4<f32>]>) {
        // Push a command to call BufferSubData()
    }
    pub fn instance_array_set_materials(&mut self, i: InstanceArrayID, start: usize, data: Box<[MaterialID]>) {
        // Push a command to call BufferSubData()
    }

    pub fn drawlist_create(&mut self, info: DrawlistInfo) -> DrawlistID {
        unimplemented!()
    }
    pub fn drawlist_delete(&mut self, id: DrawlistID) {
        unimplemented!()
    }
    pub fn drawlist_info(&mut self, id: DrawlistID) -> Option<&DrawlistInfo> {
        unimplemented!()
    }
    pub fn drawlist_set_entries(&mut self, id: DrawlistID, data: Box<[DrawlistEntry]>) {
        // Push a command to call BufferSubData() in the GL_DRAW_INDIRECT_BUFFER
        unimplemented!()
    }

    pub fn material_create(&mut self, info: MaterialInfo) -> MaterialID {
        unimplemented!()
    }
    pub fn material_delete(&mut self, id: MaterialID) {
        unimplemented!()
    }
    pub fn material_set_data(&mut self, id: MaterialID, mat: Material) {
        unimplemented!()
    }
    */

/*
    // TODO: These are problems suited for a DenseSlotMap.
    // Basically, have a singleton GpuMemory internal to G, which info is as follows:
    // - max_materials
    // - max_lights
    // - max_vertices (for vertex data buffers: position, normal, uv)
    // - max_indices
    // - max_instances (for instanced data buffers: model matrices, material indices)
    // -
    // - Indexed by MeshID (slotmap key):
    //   - vertex_mem_ranges: DenseSlotMap<Range<u32>>, (TODO: versus how many of these vertices in the range are actually used)
    //   - index_mem_ranges: DenseSlotMap<Range<u32>>, (TODO: versus how many of these indices in the range are actually used)
    // - Indexed by InstanceArrayID (slotmap key):
    //   - instance_mem_ranges: DenseSlotMap<Range<u32>>, (TODO: versus how many of these instances in the range are actually used)
    //   - instance_array_mesh: DenseSlotMap<MeshID>, (TODO: versus how many of these instances in the range are actually used)

    // TODO: Skyboxes should be specified per-viewport, not one for all viewports
    pub fn skybox_set_cubemap(&mut self, tex: CubemapSelector) {
        // Push a command to update the uniform buffer (or not ?)
    }

    pub fn light_array_set_data(&mut self, start: usize, data: Box<[Light]>) {
        // Push a command to call BufferSubData();
    }
    */
}
