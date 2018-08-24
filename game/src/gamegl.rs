use std::mem;
use std::path::{PathBuf};
use std::collections::HashMap;
use std::rc::Rc;
use fate::math::{Rgba, Rgb, Mat4, Extent2, Vec3, Vec4};
use fate::gx::{self, Object, gl::{self, types::*}, GLSLType};
use fate::img::{self, AsSlice};
use fate::font::{Font, Atlas, AtlasGlyphInfo};
use scene::{Scene, MeshID, Mesh, MeshInstance, SceneCommand, Camera};
use game::SharedGame;
use async::{Loading, Progress, Async, fs::{LoadingFile, LoadingFileProgress}};
use system::*;

static mut NB_ERRORS: usize = 0;

pub fn gl_error_hook(e: Option<gx::Error>, s: &str) {
    match e {
        Some(e) => {
            error!("GL error: {:?} ({})", e, s);
            unsafe { NB_ERRORS += 1; }
        },
        None => if unsafe { NB_ERRORS > 0 } {
            panic!("Encountered {} OpenGL errors.", unsafe { NB_ERRORS });
        }
    }
}

pub fn gl_debug_message_callback(msg: &gx::DebugMessage) {
    match ::std::ffi::CString::new(msg.text) {
        Ok(cstr) => debug!("GL: {}", cstr.to_string_lossy()),
        Err(e) => debug!("GL (UTF-8 error): {}", e),
    };
}

#[repr(u32)]
pub enum VAttrib {
    Position = 0,
    Normal = 1,
    Tangent = 2,
    BiTangent = 3,
    Color = 4,
    Uv = 5,
}

static VS_SRC: &'static [u8] = b"
#version 450 core

uniform mat4 u_proj_matrix;
uniform mat4 u_modelview_matrix;
uniform mat4 u_normal_matrix;

layout(location = 0) in vec4 a_position;
layout(location = 1) in vec4 a_normal;
layout(location = 4) in vec4 a_color;

out vec4 v_position_viewspace;
out vec4 v_normal_viewspace;
out vec4 v_color;

void main() {
    v_position_viewspace = u_modelview_matrix * vec4(a_position.xyz, 1.0);
    v_normal_viewspace = u_normal_matrix * vec4(a_normal.xyz, 0.0);
    v_color = a_color;
    gl_Position = u_proj_matrix * v_position_viewspace;
}
";
static FS_SRC: &'static [u8] = b"
#version 450 core

uniform vec3 u_light_position_viewspace;
uniform vec3 u_light_color;

in vec4 v_position_viewspace;
in vec4 v_normal_viewspace;
in vec4 v_color;

out vec4 f_color;

void main() {
    // ambient
    float ambient_strength = 0.1;
    vec3 ambient = ambient_strength * u_light_color;
  	
    // diffuse 
    vec3 norm = normalize(v_normal_viewspace.xyz);
    vec3 light_dir = normalize(u_light_position_viewspace - v_position_viewspace.xyz);
    float diff = max(0.0, dot(norm, light_dir));
    vec3 diffuse = diff * u_light_color;
    
    // specular
    float specular_strength = 0.5;
    vec3 view_dir = vec3(0.0, 0.0, -1.0);
    vec3 reflect_dir = reflect(-light_dir, norm);
    float spec = pow(max(0.0, dot(view_dir, reflect_dir)), 32);
    vec3 specular = specular_strength * spec * u_light_color;

    f_color = v_color * vec4(ambient + diffuse + specular, 1.0);
}
";

static SKY_VS_SRC: &'static [u8] = b"
#version 450 core

uniform mat4 u_proj_matrix;
uniform mat4 u_modelview_matrix;

layout(location = 0) in vec4 a_position;
layout(location = 5) in vec4 a_texcoords;

out vec3 v_tex_coords;

void main() {
    v_tex_coords = a_position.xyz;
    vec4 pos = u_proj_matrix * u_modelview_matrix * vec4(a_position.xyz, 1.0);
    gl_Position = pos.xyww; // Z = 1 after perspective divide by w
}
";

static SKY_FS_SRC: &'static [u8] = b"
#version 450 core

struct TextureSelector {
    uint tab;
    float layer;
};

uniform samplerCubeArray u_cube_map_tabs[4]; // Solid 1x1, Low-res, Medium-res, Hi-res
uniform TextureSelector u_skybox;

in vec3 v_tex_coords;

out vec4 f_color;

void main() {
    f_color = texture(u_cube_map_tabs[u_skybox.tab], vec4(v_tex_coords, u_skybox.layer));
}
";

static TEXT_VS_SRC: &'static [u8] = b"
#version 450 core

uniform mat4 u_mvp;

layout(location = 0) in vec2 a_position;
layout(location = 5) in vec2 a_tex_coords;

out vec2 v_tex_coords;

void main() {
    v_tex_coords = a_tex_coords;
    gl_Position = u_mvp * vec4(a_position, 0.0, 1.0);
}
";

static TEXT_FS_SRC: &'static [u8] = b"
#version 450 core

uniform sampler2DArray u_atlas_array;
uniform float u_atlas_index;
uniform vec4 u_color;

in vec2 v_tex_coords;

out vec4 f_color;

void main() {
    float alpha = texture(u_atlas_array, vec3(v_tex_coords, u_atlas_index)).r;

    if (alpha <= 0.01)
        discard;

    f_color = vec4(u_color.rgb, u_color.a * alpha);
}
";


fn new_gl_color_program() -> Result<gx::ProgramEx, String> {
    let vs = gx::VertexShader::try_from_source(VS_SRC)?;
    let fs = gx::FragmentShader::try_from_source(FS_SRC)?;
    let prog = gx::Program::try_from_vert_frag(&vs, &fs)?;
    Ok(gx::ProgramEx::new(prog))
}
fn new_gl_skybox_program() -> Result<gx::ProgramEx, String> {
    let vs = gx::VertexShader::try_from_source(SKY_VS_SRC)?;
    let fs = gx::FragmentShader::try_from_source(SKY_FS_SRC)?;
    let prog = gx::Program::try_from_vert_frag(&vs, &fs)?;
    Ok(gx::ProgramEx::new(prog))
}
fn new_gl_text_program() -> Result<gx::ProgramEx, String> {
    let vs = gx::VertexShader::try_from_source(TEXT_VS_SRC)?;
    let fs = gx::FragmentShader::try_from_source(TEXT_FS_SRC)?;
    let prog = gx::Program::try_from_vert_frag(&vs, &fs)?;
    Ok(gx::ProgramEx::new(prog))
}

fn unwrap_or_display_error(r: Result<gx::ProgramEx, String>) -> gx::ProgramEx {
    match r {
        Ok(p) => p,
        Err(e) => {
            error!("GL compile error\n{}", e);
            panic!("GL compile error\n{}", e)
        },
    }
}


fn gx_buffer_data<T>(target: gx::BufferTarget, data: &[T], usage: gx::BufferUsage) {
    unsafe {
        gl::BufferData(target as _, mem::size_of_val(data) as _, data.as_ptr() as _, usage as _);
    }
}
fn gx_buffer_data_dsa<T>(buf: &gx::Buffer, data: &[T], usage: gx::BufferUsage) {
    unsafe {
        gl::BindBuffer(gx::BufferTarget::Array as _, buf.gl_id());
        gx_buffer_data(gx::BufferTarget::Array, data, usage);
        gl::BindBuffer(gx::BufferTarget::Array as _, 0);
    }
}

type LoadingImg = Async<(img::Metadata, img::AnyImage), img::Error>;

#[derive(Debug)]
pub struct GLSystem {
    viewport_size: Extent2<u32>,
    color_program: gx::ProgramEx,
    skybox_program: gx::ProgramEx,
    text_program: gx::ProgramEx,
	cube_map_tabs: [gx::Texture; 2],
    files_for_2nd_cube_map_tab: HashMap<GLsizei, LoadingFile>,
    images_for_2nd_cube_map_tab: HashMap<GLsizei, LoadingImg>,
    atlas_array: gx::Texture,
    basis33_atlas_info: Rc<AtlasInfo>,
    text_mesh: TextMesh,
    mesh_vaos: HashMap<MeshID, gx::VertexArray>,
    mesh_position_buffers: HashMap<MeshID, gx::Buffer>,
    mesh_normal_buffers: HashMap<MeshID, gx::Buffer>,
    mesh_color_buffers: HashMap<MeshID, gx::Buffer>,
    mesh_index_buffers: HashMap<MeshID, gx::Buffer>,
}

fn create_1st_cube_map_tab() -> gx::Texture {
    let levels = 1;
    let level = 0;
    let internal_format = gl::RGB8;
    let format = gl::RGB;
    let type_ = gl::UNSIGNED_BYTE;
    let w = 1;
    let h = 1;
	let x = 0;
	let y = 0;
	let z = 0;
    let orange = Rgb::new(255, 175, 45);
    let pixels = [
		// Skybox 1: 6 colors
        Rgb::<u8>::new(255, 000, 000), // +X
        Rgb::<u8>::new(000, 255, 255), // -X
        Rgb::<u8>::new(000, 255, 000), // +Y
        Rgb::<u8>::new(255, 000, 255), // -Y
        Rgb::<u8>::new(000, 000, 255), // +Z
        Rgb::<u8>::new(255, 255, 000), // -Z
		// ---
        Rgb::cyan(),
        Rgb::cyan(),
        Rgb::blue(),
        Rgb::white(),
        Rgb::cyan(),
        Rgb::cyan(),
		// ---
        orange,
        orange,
        Rgb::red(),
        Rgb::yellow(),
        orange,
        orange,
		// ---
        Rgb::white(),
        Rgb::white(),
        Rgb::white(),
        Rgb::white(),
        Rgb::white(),
        Rgb::white(),
    ];
	let depth = pixels.len();
    unsafe {
        let tex = check_gl!(gx::Texture::new());
        check_gl!(gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, tex.gl_id()));
        check_gl!(gl::TexStorage3D(gl::TEXTURE_CUBE_MAP_ARRAY, levels, internal_format, w, h, depth as _));
        check_gl!(gl::TexSubImage3D(gl::TEXTURE_CUBE_MAP_ARRAY, level, x, y, z, w, h, depth as _, format, type_, pixels.as_ptr() as _));
        check_gl!(gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, 0));
        tex
    }
}

fn create_2nd_cube_map_tab(g: &G) -> (gx::Texture, HashMap<GLsizei, LoadingFile>) {
    let levels = 1;
    let internal_format = gl::RGB8;
    let w = 1024_u32;
    let h = 1024_u32;

    let dir = g.res.data_path().join(PathBuf::from("art/3rdparty/mayhem"));
    let suffixes = [ "ft", "bk", "up", "dn", "rt", "lf" ];
    let extension = "jpg";
    let mut paths = vec![];
    for name in &["grouse", "aqua4", "h2s", "flame"] {
        for suffix in &suffixes {
            paths.push(dir.join(format!("{}_{}.{}", name, suffix, extension)));
        }
    }

    for path in paths.iter() {
        info!("Checking `{}`", path.display());
        let metadata = img::load_metadata(path).unwrap();
        assert_eq!(metadata.size.w, w);
        assert_eq!(metadata.size.h, h);
        assert_eq!(metadata.pixel_format.semantic(), img::PixelSemantic::Rgb);
        assert_eq!(metadata.pixel_format.bits(), 24);
    }

    let files = paths.iter().enumerate().map(|(z, path)| {
        info!("A loading job has started for `{}`", path.display());
        (z as GLsizei, g.fs.load_file(path))
    }).collect();

    let tex = unsafe {
        let tex = check_gl!(gx::Texture::new());
        check_gl!(gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, tex.gl_id()));
        check_gl!(gl::TexStorage3D(gl::TEXTURE_CUBE_MAP_ARRAY, levels, internal_format, w as _, h as _, paths.len() as _));
        check_gl!(gl::ClearTexImage(tex.gl_id(), 0, gl::RGB, gl::UNSIGNED_BYTE, Rgb::<u8>::new(32, 110, 255).as_ptr() as _));
        check_gl!(gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, 0));
        tex
    };

    (tex, files)
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
#[repr(C)]
struct TextVertex {
    pub position: Vec2<f32>,
    pub texcoords: Vec2<f32>,
}


fn create_gl_font_atlas_array(atlas: &Atlas) -> gx::Texture {
    let levels = 1;
    let internal_format = gl::R8;
    let (w, h) = (atlas.img.width(), atlas.img.height());
    assert!(w.is_power_of_two());
    assert!(h.is_power_of_two());
    assert_eq!(w, h);

    let depth = 1; // How many elems in the array

    unsafe {
        let tex = check_gl!(gx::Texture::new());
        check_gl!(gl::BindTexture(gl::TEXTURE_2D_ARRAY, tex.gl_id()));
        check_gl!(gl::TexStorage3D(gl::TEXTURE_2D_ARRAY, levels, internal_format, w as _, h as _, depth));
        {
            let format = gl::RED;
            let type_ = gl::UNSIGNED_BYTE;
            let level = 0;
            let (x, y, z) = (0, 0, 0);
            check_gl!(gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, level, x, y, z, w as _, h as _, 1, format, type_, atlas.img.as_ptr() as _));
            info!("GL: Created font atlas array with basis33 as the first element.");
        }
        check_gl!(gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0));
        tex
    }
}

#[derive(Debug)]
struct AtlasInfo {
    glyphs: HashMap<char, AtlasGlyphInfo>,
    font_height_px: u32,
    atlas_size: Extent2<u32>,
}

impl AtlasInfo {
    pub fn new(font: &Font, atlas: &Atlas) -> Self {
        Self {
            glyphs: atlas.glyphs.clone(),
            font_height_px: font.height_px(),
            atlas_size: atlas.size(),
        }
    }
}

#[derive(Debug)]
struct TextMesh {
    vao: gx::VertexArray,
    vbo: gx::Buffer,
    ibo: gx::Buffer,
    nb_quads: usize,
    max_quads: usize,
    atlas_info: Rc<AtlasInfo>,
}

impl TextMesh {
    pub fn with_capacity(max_quads: usize, atlas_info: Rc<AtlasInfo>) -> Self {
        fn new_buffer_storage(size: usize) -> gx::Buffer {
            let buf = gx::Buffer::new();
            gx::BufferTarget::CopyRead.bind_buffer(buf.gl_id());
            gx::BufferTarget::CopyRead.set_uninitialized_buffer_storage(size, gx::BufferFlags::DYNAMIC_STORAGE);
            gx::BufferTarget::CopyRead.unbind_buffer();
            buf
        }

        let vbo = new_buffer_storage(max_quads * 4 * mem::size_of::<TextVertex>());
        let ibo = new_buffer_storage(max_quads * 6 * mem::size_of::<u16>());

        let vao = gx::VertexArray::new();
        unsafe {
            gl::BindVertexArray(vao.gl_id());
            gx::BufferTarget::Array.bind_buffer(vbo.gl_id());
            gl::EnableVertexAttribArray(VAttrib::Position as _);
            gl::EnableVertexAttribArray(VAttrib::Uv as _);
            gl::VertexAttribPointer(VAttrib::Position as _, 2, gl::FLOAT, gl::FALSE, mem::size_of::<TextVertex>() as _, 0 as _);
            gl::VertexAttribPointer(VAttrib::Uv as _, 2, gl::FLOAT, gl::FALSE, mem::size_of::<TextVertex>() as _, (2*4) as _);
            gx::BufferTarget::Array.unbind_buffer();
            gl::BindVertexArray(0);
        }

        Self {
            vbo, ibo, vao,
            nb_quads: 0,
            max_quads,
            atlas_info,
        }
    }
    pub fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao.gl_id());
            gx::BufferTarget::ElementArray.bind_buffer(self.ibo.gl_id());
            gl::DrawElements(gl::TRIANGLES, (self.nb_quads * 6) as _, gl::UNSIGNED_SHORT, ::std::ptr::null_mut());
            gx::BufferTarget::ElementArray.unbind_buffer();
            gl::BindVertexArray(0);
        }
    }
    pub fn set_text(&mut self, string: &str) {
        let &AtlasInfo {
            atlas_size, ref glyphs, font_height_px,
        } = &*self.atlas_info;

        let atlas_size = atlas_size.map(|x| x as f32);
        let mut cur = Vec2::<i16>::zero();
        let mut i = 0;

        let mut vertices = Vec::<TextVertex>::new();
        let mut indices = Vec::<u16>::new();

        self.nb_quads = 0;

        for c in string.chars() {
            match c {
                '\n' => {
                    cur.x = 0;
                    cur.y += font_height_px as i16;
                    continue;
                },
                ' ' => {
                    cur += glyphs[&' '].advance_px;
                    continue;
                },
                '\t' => {
                    cur += glyphs[&' '].advance_px * 4;
                    continue;
                },
                c if c.is_ascii_control() || c.is_ascii_whitespace() => {
                    continue;
                },
                _ => (),
            };
            let c = if glyphs.contains_key(&c) { c } else { assert!(glyphs.contains_key(&'?')); '?' };
            let glyph = &glyphs[&c];
            let mut texcoords = glyph.bounds_px.into_rect().map(
                |p| p as f32,
                |e| e as f32
            );
            texcoords.x /= atlas_size.w;
            texcoords.y /= atlas_size.h;
            texcoords.w /= atlas_size.w;
            texcoords.h /= atlas_size.h;

            let offset = glyph.bearing_px.map(|x| x as f32) / atlas_size;
            let mut world_cur = cur.map(|x| x as f32) / atlas_size;
            world_cur.y = -world_cur.y;
            world_cur.x += offset.x;
            world_cur.y -= texcoords.h - offset.y;

            let bottom_left = TextVertex {
                position: world_cur,
                texcoords: texcoords.position() + Vec2::unit_y() * texcoords.h,
            };
            let bottom_right = TextVertex {
                position: world_cur + Vec2::unit_x() * texcoords.w,
                texcoords: texcoords.position() + texcoords.extent(),
            };
            let top_left = TextVertex {
                position: world_cur + Vec2::unit_y() * texcoords.h,
                texcoords: texcoords.position(),
            };
            let top_right = TextVertex {
                position: world_cur + texcoords.extent(),
                texcoords: texcoords.position() + Vec2::unit_x() * texcoords.w,
            };

            assert!(self.nb_quads < self.max_quads);
            self.nb_quads += 1;

            vertices.push(bottom_left);
            vertices.push(bottom_right);
            vertices.push(top_left);
            vertices.push(top_right);
            indices.push(i*4 + 0);
            indices.push(i*4 + 1);
            indices.push(i*4 + 2);
            indices.push(i*4 + 3);
            indices.push(i*4 + 2);
            indices.push(i*4 + 1);

            cur += glyph.advance_px;
            i += 1;
        }

        gx::BufferTarget::Array.bind_buffer(self.vbo.gl_id());
        gx::BufferTarget::Array.set_buffer_subdata::<TextVertex>(&vertices, 0);
        gx::BufferTarget::Array.unbind_buffer();

        gx::BufferTarget::ElementArray.bind_buffer(self.ibo.gl_id());
        gx::BufferTarget::ElementArray.set_buffer_subdata::<u16>(&indices, 0);
        gx::BufferTarget::ElementArray.unbind_buffer();
    }
}

impl GLSystem {
    pub fn new(viewport_size: Extent2<u32>, g: &SharedGame) -> Self {
        let basis33_atlas_info = Rc::new(AtlasInfo::new(g.res.basis33(), g.res.basis33_atlas()));
        let text_mesh = TextMesh::with_capacity(1024, basis33_atlas_info.clone());

        let (cube_map_tab_2, files_for_2nd_cube_map_tab) = create_2nd_cube_map_tab(g);

        Self {
            viewport_size,
            color_program:  unwrap_or_display_error(new_gl_color_program()),
            skybox_program: unwrap_or_display_error(new_gl_skybox_program()),
            text_program:   unwrap_or_display_error(new_gl_text_program()),
            atlas_array: create_gl_font_atlas_array(g.res.basis33_atlas()),
            files_for_2nd_cube_map_tab,
            images_for_2nd_cube_map_tab: HashMap::new(),
            cube_map_tabs: [create_1st_cube_map_tab(), cube_map_tab_2],
            basis33_atlas_info,
            text_mesh,
            mesh_vaos: Default::default(),
            mesh_position_buffers: Default::default(),
            mesh_normal_buffers: Default::default(),
            mesh_color_buffers: Default::default(),
            mesh_index_buffers: Default::default(),
        }
    }

    fn render_scene(&mut self, scene: &Scene, draw: &Draw) {
        for camera in scene.cameras.values() {
            unsafe {
                let Extent2 { w, h } = camera.viewport_size;
                gl::Viewport(0, 0, w as _, h as _); // XXX x and y are mindlessly set to zero
            }
            self.render_scene_with_camera(scene, draw, camera);
            self.render_skybox(scene, draw, camera);
        }
        // Alpha-blended; do last
        self.render_text(draw, &scene.gui_camera);
    }

    fn render_text(&mut self, _draw: &Draw, camera: &Camera) {
        let texture_unit: i32 = 9;
        unsafe {
            gl::UseProgram(self.text_program.inner().gl_id());
            gl::ActiveTexture(gl::TEXTURE0 + texture_unit as u32);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.atlas_array.gl_id());
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
            //gl::Disable(gl::DEPTH_TEST);
        }

        self.text_program.set_uniform_primitive("u_atlas_index", &[0 as f32]);
        self.text_program.set_uniform("u_atlas_array", GLSLType::Sampler2DArray, &[texture_unit]);

        for i in 0..2 {
            let mvp = {
                let position_viewport_space = Vec2::new(4, self.basis33_atlas_info.font_height_px as i32) + i;
                let Extent2 { w, h } = self.basis33_atlas_info.atlas_size
                    .map(|x| x as f32) * 2. / camera.viewport_size.map(|x| x as f32);
                let t = camera.viewport_to_ugly_ndc(position_viewport_space);
                Mat4::<f32>::translation_3d(t) * Mat4::scaling_3d(Vec3::new(w, h, 1.))
            };

            let color = if i == 0 {
                Rgba::new(1., 4., 0., 1_f32)
            } else {
                Rgba::black()
            };

            self.text_program.set_uniform_primitive("u_mvp", &[mvp]);
            self.text_program.set_uniform_primitive("u_color", &[color]);

            self.text_mesh.draw();
        }


        unsafe {
            //gl::Enable(gl::DEPTH_TEST);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::UseProgram(0);
        }
    }

    fn render_skybox(&mut self, scene: &Scene, _draw: &Draw, camera: &Camera) {
        let mesh_id = &Scene::MESHID_SKYBOX;
        let mesh = &scene.meshes[mesh_id];

        let view = camera.view_matrix();
        let proj = camera.proj_matrix();
        let view_without_translation = {
            let mut r = view;
            r.cols.w = Vec4::unit_w();
            r
        };

        let funny: i32 = 9; // Important: Use i32, not u32.
        unsafe {
            gl::UseProgram(self.skybox_program.inner().gl_id());

            for (i, cube_map_tab) in self.cube_map_tabs.iter().enumerate() {
                gl::ActiveTexture(gl::TEXTURE0 + funny as u32 + i as u32);
                gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, cube_map_tab.gl_id());
                // FIXME: Be less braindead and use sampler objects
                gl::TexParameteri(gl::TEXTURE_CUBE_MAP_ARRAY, gl::TEXTURE_MAG_FILTER, scene.skybox_min_mag_filter as _);
                gl::TexParameteri(gl::TEXTURE_CUBE_MAP_ARRAY, gl::TEXTURE_MIN_FILTER, scene.skybox_min_mag_filter as _);
            }

            gl::BindVertexArray(self.mesh_vaos[mesh_id].gl_id()); // FIXME: Filling them every time = not efficient
            gl::DepthFunc(gl::LEQUAL);
        }

        self.skybox_program.set_uniform_primitive("u_proj_matrix", &[proj]);
        self.skybox_program.set_uniform_primitive("u_modelview_matrix", &[view_without_translation]);
        {
            let tabs = self.skybox_program.uniform("u_cube_map_tabs[0]").unwrap();
            assert_eq!(tabs.type_, Some(GLSLType::SamplerCubeMapArray));
            assert_eq!(tabs.array_len, 4);
            self.skybox_program.set_uniform_unchecked(tabs.location, &[funny, funny+1, funny, funny+1]);

            assert!((scene.skybox_selector.tab as i32) < tabs.array_len);
            self.skybox_program.set_uniform_primitive("u_skybox.tab", &[scene.skybox_selector.tab as u32]);
            self.skybox_program.set_uniform_primitive("u_skybox.layer", &[scene.skybox_selector.layer as f32]);
        }

        self.gl_update_mesh_position_attrib(mesh_id, mesh);
        self.gl_draw_mesh(mesh_id, mesh);

        unsafe {
            gl::DepthFunc(gl::LESS);
            gl::BindVertexArray(0);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, 0);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::UseProgram(0);
        }
    }

    fn render_scene_with_camera(&mut self, scene: &Scene, _draw: &Draw, camera: &Camera) {
        let view = camera.view_matrix();
        let proj = camera.proj_matrix();
        
        unsafe {
            gl::UseProgram(self.color_program.inner().gl_id());
        }

        self.color_program.set_uniform_primitive("u_proj_matrix", &[proj]);
        self.color_program.set_uniform_primitive("u_light_position_viewspace", &[Vec3::new(0., 0., 0.)]);
        self.color_program.set_uniform_primitive("u_light_color", &[Rgb::white()]);

        for &MeshInstance { ref mesh_id, xform } in scene.mesh_instances.values() {
            let mesh = &scene.meshes[mesh_id];
            let model = Mat4::from(xform);
            let modelview = view * model;
            let normal_matrix = modelview.inverted().transposed();
            self.color_program.set_uniform_primitive("u_modelview_matrix", &[modelview]);
            self.color_program.set_uniform_primitive("u_normal_matrix", &[normal_matrix]);

            unsafe {
                gl::BindVertexArray(self.mesh_vaos[mesh_id].gl_id()); // FIXME: Filling them every time = not efficient
            }

            self.gl_update_mesh_position_attrib(mesh_id, mesh);
            self.gl_update_mesh_normal_attrib(mesh_id, mesh);
            self.gl_update_mesh_color_attrib(mesh_id, mesh);
            self.gl_draw_mesh(mesh_id, mesh);

            unsafe {
                gl::BindVertexArray(0);
            }
        }
        unsafe {
            gl::UseProgram(0);
        }
    }
    fn gl_update_mesh_position_attrib(&self, mesh_id: &MeshID, mesh: &Mesh) {
        assert!(!mesh.vposition.is_empty());
        let pos_buffer = self.mesh_position_buffers.get(mesh_id).expect("Meshes must have a position buffer (for now)!");
        unsafe {
            gl::BindBuffer(gx::BufferTarget::Array as _, pos_buffer.gl_id());
            gl::EnableVertexAttribArray(VAttrib::Position as _);
            gl::VertexAttribPointer(VAttrib::Position as _, 4, gl::FLOAT, gl::FALSE, 4*4, 0 as _);
            gl::BindBuffer(gx::BufferTarget::Array as _, 0);
        }
    }
    fn gl_update_mesh_normal_attrib(&self, mesh_id: &MeshID, mesh: &Mesh) {
        assert!(!mesh.vnormal.is_empty());
        let norm_buffer = self.mesh_normal_buffers.get(mesh_id).expect("Meshes must have a normals buffer (for now)!");
        unsafe {
            gl::BindBuffer(gx::BufferTarget::Array as _, norm_buffer.gl_id());
            gl::EnableVertexAttribArray(VAttrib::Normal as _);
            gl::VertexAttribPointer(VAttrib::Normal as _, 4, gl::FLOAT, gl::FALSE, 4*4, 0 as _);
            gl::BindBuffer(gx::BufferTarget::Array as _, 0);
        }
    }
    fn gl_update_mesh_color_attrib(&self, mesh_id: &MeshID, mesh: &Mesh) {
        let set_default_color = |rgba: Rgba<u8>| unsafe {
            let rgba = rgba.map(|x| x as f32) / 255.;
            gl::DisableVertexAttribArray(VAttrib::Color as _);
            gl::VertexAttrib4f(VAttrib::Color as _, rgba.r, rgba.g, rgba.b, rgba.a);
        };
        match self.mesh_color_buffers.get(mesh_id) {
            None => set_default_color(Rgba::white()),
            Some(col_buffer) => {
                match mesh.vcolor.len() {
                    0 => set_default_color(Rgba::white()),
                    1 => set_default_color(mesh.vcolor[0]),
                    _ => unsafe {
                        gl::BindBuffer(gx::BufferTarget::Array as _, col_buffer.gl_id());
                        gl::EnableVertexAttribArray(VAttrib::Color as _);
                        gl::VertexAttribPointer(VAttrib::Color as _, 4, gl::FLOAT, gl::TRUE, 4, 0 as _);
                        gl::BindBuffer(gx::BufferTarget::Array as _, 0);
                    },
                }
            },
        }
    }
    fn gl_draw_mesh(&self, mesh_id: &MeshID, mesh: &Mesh) {
        if let Some(idx_buffer) = self.mesh_index_buffers.get(mesh_id) {
            if !mesh.indices.is_empty() {
                unsafe {
                    gl::BindBuffer(gx::BufferTarget::ElementArray as _, idx_buffer.gl_id());
                    assert_eq!(mem::size_of_val(&mesh.indices[0]), 2); // for gl::UNSIGNED_SHORT
                    gl::DrawElements(mesh.topology, mesh.indices.len() as _, gl::UNSIGNED_SHORT, 0 as _);
                    gl::BindBuffer(gx::BufferTarget::ElementArray as _, 0);
                }
            }
        } else {
            unsafe {
                gl::DrawArrays(mesh.topology, 0, mesh.vposition.len() as _);
            }
        }
    }
    fn pump_scene_draw_commands(&mut self, scene: &mut Scene) {
        for cmd in scene.draw_commands_queue.iter() {
            self.handle_scene_command(scene, cmd);
        }
    }
    fn handle_scene_command(&mut self, scene: &Scene, cmd: &SceneCommand) {
        match *cmd {
            SceneCommand::AddMesh(mesh_id) => {
                if let Some(&Mesh { topology: _, ref vposition, ref vnormal, ref vcolor, ref indices, }) = scene.meshes.get(&mesh_id) {
                    self.mesh_vaos.entry(mesh_id).or_insert_with(gx::VertexArray::new);
                    gx_buffer_data_dsa(self.mesh_position_buffers.entry(mesh_id).or_insert_with(gx::Buffer::new), vposition, gx::BufferUsage::StaticDraw);
                    gx_buffer_data_dsa(self.mesh_normal_buffers.entry(mesh_id).or_insert_with(gx::Buffer::new), vnormal, gx::BufferUsage::StaticDraw);
                    if vcolor.is_empty() {
                        self.mesh_color_buffers.remove(&mesh_id);
                    } else {
                        gx_buffer_data_dsa(self.mesh_color_buffers.entry(mesh_id).or_insert_with(gx::Buffer::new), vcolor, gx::BufferUsage::StaticDraw);
                    }
                    if indices.is_empty() {
                        self.mesh_index_buffers.remove(&mesh_id);
                    } else {
                        gx_buffer_data_dsa(self.mesh_index_buffers.entry(mesh_id).or_insert_with(gx::Buffer::new), indices, gx::BufferUsage::StaticDraw);
                    }
                }
            },
            SceneCommand::AddMeshInstance(_id) => {},
        }
    }
}

impl System for GLSystem {
    fn on_canvas_resized(&mut self, _g: &mut G, size: Extent2<u32>) {
        self.viewport_size = size;
    }
    fn draw(&mut self, g: &mut G, d: &Draw) {
        unsafe {
            let Extent2 { w, h } = self.viewport_size;
            gl::Viewport(0, 0, w as _, h as _);
            gl::ClearColor(1., 0., 1., 1.);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let mut text = match g.last_fps_stats() {
            Some(fps_stats) => format!("{} FPS", fps_stats.fps()),
            None => format!("(No FPS stats available yet)"),
        };
        text += "\nHello, text world!\n\n";

        let mut completed = vec![];
        for (z, file) in self.files_for_2nd_cube_map_tab.iter() {
            match file.poll() {
                LoadingFileProgress::Pending => continue,
                LoadingFileProgress::Loading { nb_bytes_read, total_nb_bytes, thread_id } => {
                    let progress = nb_bytes_read as f32 / total_nb_bytes as f32;
                    text += &format!("Loading `{}`... {}% (Thread {})\n", file.path().display(), (100. * progress).round() as u32, thread_id);
                },
                LoadingFileProgress::Complete { .. } => {
                    completed.push(*z);
                },
            }
        }
        let datas: Vec<_> = completed.into_iter().map(|z| {
            let file = self.files_for_2nd_cube_map_tab.remove(&z).unwrap();
            let data = file.wait().unwrap();
            (z, data)
        }).collect();
        self.images_for_2nd_cube_map_tab.extend(datas.into_iter().map(|(z, data)| {
            (z, g.mt.do_async(Box::new(|| img::load_from_memory(data))))
        }));

        let mut completed = vec![];
        for (z, img) in self.images_for_2nd_cube_map_tab.iter() {
            if img.poll().is_complete() {
                completed.push(*z);
            }
        }
        let cube_map_tab_2 = self.cube_map_tabs[1].gl_id();
        for (z, img) in completed.into_iter().map(|z| (z, self.images_for_2nd_cube_map_tab.remove(&z).unwrap())) {
            match img.wait() {
                Ok((_, img::AnyImage::Rgb8(img))) => {
                    let level = 0;
                    let format = gl::RGB;
                    let type_ = gl::UNSIGNED_BYTE;
                    let (x, y, w, h) = (0, 0, 1024, 1024); // XXX
                    unsafe {
                        check_gl!(gl::TextureSubImage3D(cube_map_tab_2, level, x, y, z, w, h, 1, format, type_, img.as_ptr() as _));
                    }
                },
                _ => unimplemented!{},
            }
        }

        self.text_mesh.set_text(&text);

        self.pump_scene_draw_commands(&mut g.scene);
        self.render_scene(&mut g.scene, d);
    }
}

