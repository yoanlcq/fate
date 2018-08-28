// Domain entity (i.e things that exist in the game world)
// - Transform
// - Camera
// - ModelInstance
//
// Domain 
//
// Domain texture
// - CubemapArray
// - Texture2DArray

macro_rules! id_type {
    ($($ID:ident)+) => { $(id_type!{@ $ID})+ };
    (@ $ID:ident) => {
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $ID(u32);
    };
}

id_type!{EID ModelID}

#[derive(Debug, Clone, PartialEq)]
pub struct DB {
    // For renderers
    cmd_queue: VecDeque<Cmd>,

    // "singletons"
    clear_color: Rgba<f32>,
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
}

pub enum XformParent {
    // Normalized Device Coordinates, no matter the camera.
    NDC,
    // 3D/2D world coordinates.
    World,
    //
    EID(EID),
}

pub struct Xform {
    pub parent: XformParent,
    // pos, orientation, scale...
}

pub struct ViewportNode {
    parent: Option<ViewportNodeID>,
    split: Option<SplitDirection>,
    v0: ViewportInfo,
    v1: ViewportInfo,
}

pub struct ViewportInfo {
    visual_layers: HashSet<VisualLayerID>,
    camera: CameraID,
}


pub struct ModelInstance {
    model_id: ModelID,
    // TODO: Materials??
}
pub struct PlaneInstance {
    plane_id: PlaneID,
    texture: Texture2DSelector,
}


// Commands for the rendering backend to update the on-GPU data.
enum Cmd {
    ClearColorEdit,
    SkyboxToggled,
    SkyboxCubemapSelected,
    CubemapArrayCreate(CubemapArrayID),
    CubemapArrayDelete(CubemapArrayID),
    CubemapArrayClear(CubemapArrayID),
    CubemapArraySubImage(CubemapArrayID, CubemapArrayImageKey),
    Texture2DArrayCreate(Texture2DArrayID),
    Texture2DArrayDelete(Texture2DArrayID),
    Texture2DArrayClear(Texture2DArrayID),
    Texture2DArraySubImage(Texture2DArrayID, Texture2DArrayImageKey),
    ModelCreate(ModelID),
    ModelDelete(ModelID),
    ModelEditPositions(ModelID, Range<u32>),
    ModelEditNormals(ModelID, Range<u32>),
    ModelEditColors(ModelID, Range<u32>),
    ModelEditIndices(ModelID, Range<u32>),
    PlaneCreate(PlaneID),
    PlaneDelete(PlaneID),
    PlaneEditPositions(PlaneID, Range<u32>),
    PlaneEditUVs(PlaneID, Range<u32>),
    PlaneEditColors(PlaneID, Range<u32>),
    PlaneEditIndices(PlaneID, Range<u32>),
}

pub struct Texture2DArrayInfo {
    pub nb_levels: u32,
    pub internal_format: TextureInternalFormat, // e.g GL_RGB8
    pub size: Extent2<u32>,
    pub nb_slots: u32,
    pub clear_color: Rgba<f32>,

    /// If false, the EndSystem drops the heavyweight data. Keeping in CPU is useful for further processing.
    pub keep_in_cpu: bool,
    pub images: HashMap<Texture2DArrayImageKey, Img>,
}

pub struct Texture2DArrayImageKey {
    pub level: u32,
    pub index: u32,
}

pub struct Texture2DSelector {
    pub array: Texture2DArrayID,
    pub slot: u32,
}


pub struct CubemapSelector {
    pub array: CubemapArrayID,
    pub slot: u32,
}

pub struct CubemapArrayInfo {
    pub nb_levels: u32,
    pub internal_format: TextureInternalFormat, // e.g GL_RGB8
    pub size: Extent2<u32>,
    pub nb_cubemaps: u32,
    pub clear_color: Rgba<f32>,

    /// If false, the EndSystem drops the heavyweight data. Keeping in CPU is useful for further processing.
    pub keep_in_cpu: bool,
    pub images: HashMap<CubemapArrayImageKey, Img>,
}

pub struct CubemapArrayImageKey {
    pub level: u32,
    pub cubemap_index: u32,
    pub face: CubemapFace, // +X, -X, etc
}

pub struct ModelInfo {
    // Also used by renderers
    pub topology: Topology,
    pub nb_vertices: u32,
    pub nb_indices: u32,

    /// If false, the EndSystem drops the heavyweight data. Keeping in CPU is useful for further processing.
    pub keep_in_cpu: bool,
    pub v_position: Vec<Vec3<f32>>, // Not optional
    pub v_normal: Vec<Vec3<f32>>, // Not optional
    pub v_color: Vec<Rgba<u8>>, // Optional. If there's only one element, it is used for all vertices.
    pub indices: Either<Vec<u16>, Vec<u32>>, // Optional. If empty, it's rendered using glDrawArrays.
}

pub struct PlaneInfo {
    // Also used by renderers
    pub topology: Topology,
    pub nb_vertices: u32,
    pub nb_indices: u32,

    /// If false, the EndSystem drops the heavyweight data. Keeping in CPU is useful for further processing.
    pub keep_in_cpu: bool,
    pub v_position: Vec<Vec2<f32>>, // Not optional
    pub v_uv: Vec<Vec2<f32>>,
    pub v_texture2d_selector: Vec<RawTexture2DSelector>, // If there's only one element, it is used for all vertices.
    pub v_color: Vec<Rgba<u8>>, // Optional. If there's only one element, it is used for all vertices.
    pub indices: Vec<u16>, // Optional. If empty, it's rendered using glDrawArrays.
}


#[derive(Debug, Clone, PartialEq)]
pub struct GLDB {
    va: HashMap<ModelID, gx::VertexArray>,
    vb_position: HashMap<ModelID, gx::Buffer>,
    vb_normal: HashMap<ModelID, gx::Buffer>,
    vb_color: HashMap<ModelID, gx::Buffer>,
    vb_indices: HashMap<ModelID, gx::Buffer>,
}

