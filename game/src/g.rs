use std::time::Duration;
use std::collections::{VecDeque, HashMap};
use std::sync::Arc;

use rand::random;

use fate::mt;
use fate::math::{Extent2, Rgba};
use fate::lab::fps::FpsStats;

use frame_time::FrameTimeManager;
use message::Message;
use input::Input;
use resources::Resources;
use gpu::GpuCmd;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SplitOrigin {
    LeftOrBottom, Middle, RightOrTop,    
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SplitUnit {
    Ratio, Px,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal, Vertical,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Split {
    pub origin: SplitOrigin,
    pub unit: SplitUnit,
    pub value: f32,
    pub direction: SplitDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewportNode {
    Whole {
        parent: Option<ViewportNodeID>,
        info: ViewportInfo
    },
    Split {
        parent: Option<ViewportNodeID>,
        split: Split,
        children: (ViewportNodeID, ViewportNodeID),
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ViewportInfo {
    // TODO: Describes what a viewport displays    
    pub clear_color: Rgba<f32>,
}

impl Default for ViewportNode {
    fn default() -> Self {
        ViewportNode::Whole {
            parent: None,
            info: Default::default(),
        }
    }
}

macro_rules! id_type {
    ($($ID:ident)+) => { $(id_type!{@ $ID})+ };
    (@ $ID:ident) => {
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $ID(u32);
    };
}


id_type!{ViewportNodeID}

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
    clear_color: Rgba<f32>,
    _highest_viewport_node_id: ViewportNodeID, // Do not keep; Replace by SlotMap!
    root_viewport_node_id: ViewportNodeID,
    focused_viewport_node_id: ViewportNodeID,
    hovered_viewport_node_id: ViewportNodeID,
    viewport_nodes: HashMap<ViewportNodeID, ViewportNode>,
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
    model_infos: HashMap<ModelID, ModelInfo>,
    plane_infos: HashMap<PlaneID, PlaneInfo>,

    //
    cubemap_arrays: HashMap<CubemapArrayID, CubemapArrayInfo>,

    //
    texture2d_arrays: HashMap<Texture2DArrayID, Texture2DArrayInfo>,

    viewport_nodes: HashMap<ViewportNodeID, ViewportNode>,
    */
}


impl G {
    pub fn new(canvas_size: Extent2<u32>, mt: Arc<mt::SharedThreadContext>) -> Self {

        let mut viewport_nodes = HashMap::new();
        let root_viewport_node_id = ViewportNodeID(0);
        viewport_nodes.insert(root_viewport_node_id, ViewportNode::default());
        let _highest_viewport_node_id = root_viewport_node_id;

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
            viewport_nodes,
            _highest_viewport_node_id,
            root_viewport_node_id,
            focused_viewport_node_id: root_viewport_node_id,
            hovered_viewport_node_id: root_viewport_node_id,
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
    pub fn root_viewport_node_id(&self) -> ViewportNodeID {
        self.root_viewport_node_id
    }
    pub fn focused_viewport_node_id(&self) -> ViewportNodeID {
        self.focused_viewport_node_id
    }
    pub fn hovered_viewport_node_id(&self) -> ViewportNodeID {
        self.hovered_viewport_node_id
    }
    pub fn viewport_node(&self, id: ViewportNodeID) -> Option<&ViewportNode> {
        self.viewport_nodes.get(&id)
    }
    pub fn viewport_node_mut(&mut self, id: ViewportNodeID) -> Option<&mut ViewportNode> {
        self.viewport_nodes.get_mut(&id)
    }
    pub fn viewport_split_h(&mut self) {
        self.viewport_split(SplitDirection::Horizontal)
    }
    pub fn viewport_split_v(&mut self) {
        self.viewport_split(SplitDirection::Vertical)
    }
    pub fn viewport_split(&mut self, direction: SplitDirection) {
        let id = self.focused_viewport_node_id();

        let c0_id = ViewportNodeID(self._highest_viewport_node_id.0 + 1);
        let c1_id = ViewportNodeID(self._highest_viewport_node_id.0 + 2);

        let info = {
            let node = self.viewport_node_mut(id).unwrap();
            let (parent, info) = match *node {
                ViewportNode::Split { .. } => panic!("A non-leaf viewport node cannot be focused"),
                ViewportNode::Whole { ref info, parent, .. } => (parent, info.clone()),
            };
            *node = ViewportNode::Split {
                parent,
                children: (c0_id, c1_id),
                split: Split {
                    direction,
                    origin: SplitOrigin::Middle,
                    unit: SplitUnit::Ratio,
                    value: 0.,
                }
            };
            info
        };

        self._highest_viewport_node_id.0 += 2;
        let c0_info = info.clone();
        let mut c1_info = info;

        self.focused_viewport_node_id = c0_id;
        c1_info.clear_color = Rgba::<u8>::new_opaque(random(), random(), random()).map(|x| x as f32 / 255.);

        let c0_node = ViewportNode::Whole { info: c0_info, parent: Some(id) };
        let c1_node = ViewportNode::Whole { info: c1_info, parent: Some(id) };
        self.viewport_nodes.insert(c0_id, c0_node);
        self.viewport_nodes.insert(c1_id, c1_node);
    }
    /// Merges the focused viewport node into its neighbour.
    pub fn viewport_merge(&mut self) {
        let focus_id = self.focused_viewport_node_id();

        let (merge_id, info) = {
            let focus = self.viewport_node_mut(focus_id).unwrap();
            let (parent, info) = match *focus {
                ViewportNode::Split { .. } => panic!("A non-leaf viewport node cannot be focused"),
                ViewportNode::Whole { parent, ref info } => (parent, info.clone()),
            };
            (parent, info)
        };

        let merge_id = match merge_id {
            None => return,
            Some(x) => x,
        };

        let (c0_id, c1_id) = {
            let merge = self.viewport_node_mut(merge_id).unwrap();
            let (parent, c0_id, c1_id) = match *merge {
                ViewportNode::Whole { .. } => panic!("A parent node can't be whole"),
                ViewportNode::Split { parent, children, .. } => (parent, children.0, children.1),
            };
            *merge = ViewportNode::Whole { info, parent };
            (c0_id, c1_id)
        };

        self.viewport_nodes.remove(&c0_id).unwrap();
        self.viewport_nodes.remove(&c1_id).unwrap();
        self.focused_viewport_node_id = merge_id;
    }
}
