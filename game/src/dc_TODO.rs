// WIP file, unused!

struct DeviceContext {
    
}

impl DeviceContext {
    // Low-level foundations
    pub fn cubemap_create(&self, size: Extent2<u32>, levels: u32, format: Format) -> CubemapID { unimplemented![] }
    pub fn cubemap_clear(&self, id: CubemapID, face: CubemapFace, color: Rgba<f32>) { unimplemented![] }
    pub fn cubemap_set(&self, id: CubemapID, face: CubemapFace, level: u32, img: Img<Rgb<u8>>) { unimplemented![] }
    pub fn cubemap_destroy(&self, id: CubeMapID) { unimplemented![] }
    pub fn skybox_create(&self) { unimplemented!{} }
    pub fn skybox_destroy(&self) { unimplemented!{} }
    pub fn skybox_set_cube_map(&self, cube_map: CubeMapID) { unimplemented!{} }
    pub fn skybox_set_visible(&self, do_use: bool) { unimplemented!{} }
    pub fn skybox_is_visible(&self) -> bool { unimplemented!{} }
    pub fn buffer_create(&self, cap: usize, flags: BufferFlags) -> BufferID { unimplemented!{} }
    pub fn buffer_destroy(&self, id: BufferID) { unimplemented!{} }
    pub fn buffer_set_data<T>(&self, id: BufferID, data: &[T]) { unimplemented!{} }
    pub fn fbo; // create, destroy, blit, etc...
    pub fn texture2d; // set filters, etc
    pub fn shader; // set uniform, etc
    pub fn layout;
    pub fn draw_elements;
    pub fn pipeline;
    pub fn dispatch_compute; // ...
    pub fn fence;
    pub fn barrier;
    
    // Position, Normal, Tangent, BiTangent, Color, Uv
    pub fn va_create(&self) -> MeshID;
    pub fn va_destroy(&self, id: MeshID); 
    // Also used to determine the number of vertices.
    pub fn va_set_positions(&self, id: VaID, v: &[Vec3<f32>]);
    pub fn va_set_normals(&self, id: VaID, v: &[Vec3<f32>]);
    pub fn va_set_tangents(&self, id: VaID, v: &[Vec3<f32>]);
    pub fn va_set_bitangents(&self, id: VaID, v: &[Vec3<f32>]);
    pub fn va_set_colors(&self, id: VaID, v: &[Vec4<u8>]);
    pub fn va_set_uvs(&self, id: VaID, v: &[Vec2<f32>]);
    pub fn va_set_material(&self, id: VaID, mat: MaterialID);
    pub fn va_instantiate(&self, id: VaID) -> InstanceID;
    pub fn instance_set_xform(&self, id: InstanceID);
    pub fn instance_set_material(&self, mat: MaterialID);
    
    // Camera, viewport, etc
    
    pub fn push_color(&self);
    pub fn pop_color(&self);
    pub fn push_font(&self);
    pub fn pop_font(&self);
    
    pub fn layer2d_create(&self); 
    pub fn layer2d_destroy(&self);
    pub fn layer2d_add_text(&self);
    pub fn layer2d_add_quads(&self);
    pub fn layer2d_add_text_styled(&self);
    
    // Immediate drawing functions, based upon lower layers
    pub fn im_draw_text(&self);
    pub fn im_draw_quads(&self);
    pub fn im_draw_line(&self);
    pub fn im_draw_mesh(&self);
}
