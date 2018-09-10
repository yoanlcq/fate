use std::time::Duration;
use std::collections::{VecDeque, HashMap};
use std::sync::Arc;
use std::ops::Range;

use fate::mt;
use fate::math::{Extent2, Rgba, Rect};
use fate::lab::fps::FpsStats;

use frame_time::FrameTimeManager;
use message::Message;
use input::Input;
use resources::Resources;
use gpu::{GpuCmd, CpuSubImage2D};
use mouse_cursor::MouseCursor;
use viewport::{ViewportDB, ViewportVisitor};
use cubemap::{CubemapArrayInfo, CubemapArrayID, CubemapFace};
use texture2d::{Texture2DArrayInfo, Texture2DArrayID};
use mesh::{MeshID, MeshInfo};

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

    //
    cubemap_arrays: [CubemapArrayInfo; CubemapArrayID::MAX],
    texture2d_arrays: [Texture2DArrayInfo; Texture2DArrayID::MAX],

    mesh_infos: HashMap<MeshID, MeshInfo>,

    /*
    skybox_is_enabled: bool,
    skybox_cubemap_selector: CubemapSelector,

    // "entities"
    xforms: HashMap<EID, Xform>,
    cameras: HashMap<EID, Camera>,
    models: HashMap<EID, ModelInstance>,
    planes: HashMap<EID, PlaneInstance>,
    visual_layers: HashMap<EID, VisualLayerID>,
    physics_layers: HashMap<EID, PhysicsLayerID>,
    visual_space: HashMap<EID, VisualSpace>,

    //
    plane_infos: HashMap<PlaneID, PlaneInfo>,
    */
}

impl G {
    pub fn new(canvas_size: Extent2<u32>, mt: Arc<mt::SharedThreadContext>) -> Self {
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
            viewport_db: ViewportDB::new(),
            cubemap_arrays: array![CubemapArrayInfo::new(); CubemapArrayID::MAX],
            texture2d_arrays: array![Texture2DArrayInfo::new(); Texture2DArrayID::MAX],
            mesh_infos: HashMap::new(),
        };
        g.gpu_cmd_queue.push_back(GpuCmd::ClearColorEdit);
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

    pub fn viewport_db(&self) -> &ViewportDB {
        &self.viewport_db
    }
    pub fn viewport_db_mut(&mut self) -> &mut ViewportDB {
        &mut self.viewport_db
    }
    pub fn visit_viewports(&mut self, f: &mut ViewportVisitor) {
        let Extent2 { w, h } = self.input.canvas_size();
        self.viewport_db_mut().visit(Rect { x: 0, y: 0, w, h }, f);
    }

    pub fn cubemap_array_info(&self, array: CubemapArrayID) -> Option<&CubemapArrayInfo> {
        self.cubemap_arrays.get(array.0 as usize)
    }
    pub fn cubemap_array_info_mut(&mut self, array: CubemapArrayID) -> Option<&mut CubemapArrayInfo> {
        self.cubemap_arrays.get_mut(array.0 as usize)
    }
    pub fn cubemap_array_create(&mut self, array: CubemapArrayID) {
        self.gpu_cmd_queue.push_back(GpuCmd::CubemapArrayCreate(array))
    }
    pub fn cubemap_array_delete(&mut self, array: CubemapArrayID) {
        self.gpu_cmd_queue.push_back(GpuCmd::CubemapArrayDelete(array))
    }
    pub fn cubemap_array_clear(&mut self, array: CubemapArrayID, level: u32, color: Rgba<f32>) {
        self.gpu_cmd_queue.push_back(GpuCmd::CubemapArrayClear(array, level, color))
    }
    pub fn cubemap_array_sub_image_2d(&mut self, array: CubemapArrayID, cubemap: usize, face: CubemapFace, img: CpuSubImage2D) {
        self.gpu_cmd_queue.push_back(GpuCmd::CubemapArraySubImage2D(array, cubemap, face, img))
    }

    pub fn texture2d_array_info(&self, array: Texture2DArrayID) -> Option<&Texture2DArrayInfo> {
        self.texture2d_arrays.get(array.0 as usize)
    }
    pub fn texture2d_array_info_mut(&mut self, array: Texture2DArrayID) -> Option<&mut Texture2DArrayInfo> {
        self.texture2d_arrays.get_mut(array.0 as usize)
    }
    pub fn texture2d_array_create(&mut self, array: Texture2DArrayID) {
        self.gpu_cmd_queue.push_back(GpuCmd::Texture2DArrayCreate(array))
    }
    pub fn texture2d_array_delete(&mut self, array: Texture2DArrayID) {
        self.gpu_cmd_queue.push_back(GpuCmd::Texture2DArrayDelete(array))
    }
    pub fn texture2d_array_clear(&mut self, array: Texture2DArrayID, level: u32, color: Rgba<f32>) {
        self.gpu_cmd_queue.push_back(GpuCmd::Texture2DArrayClear(array, level, color))
    }
    pub fn texture2d_array_sub_image_2d(&mut self, array: Texture2DArrayID, slot: usize, img: CpuSubImage2D) {
        self.gpu_cmd_queue.push_back(GpuCmd::Texture2DArraySubImage2D(array, slot, img))
    }

    /*
    // 1 mesh3d = 1 VAO
    pub fn mesh3d_info(&self, id: Mesh3DID) -> Option<&Mesh3DInfo> {
        self.mesh3d_infos.get(&id)
    }
    pub fn mesh3d_info_mut(&mut self, id: Mesh3DID) -> Option<&mut Mesh3DInfo> {
        self.mesh3d_infos.get_mut(&id)
    }
    pub fn mesh3d_create(&mut self, id: Mesh3DID) {
        unimplemented!()
    }
    pub fn mesh3d_delete(&mut self, id: Mesh3DID) {
        unimplemented!()
    }
    pub fn mesh3d_update_channel(&mut self, id: Mesh3DID, channel: Mesh3DChannel, range: Range<usize>) {
        unimplemented!()
    }
    */
}
