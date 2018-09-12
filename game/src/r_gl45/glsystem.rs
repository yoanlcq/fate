use fate::math::{Extent2, Rgba, Rect};
use fate::gx::{self, gl::{self, types::*}};

use gpu::GpuCmd;
use viewport::{ViewportVisitor, AcceptLeafViewport};
use cubemap::{CubemapArrayID};
use texture2d::Texture2DArrayID;
use mesh::VertexAttribIndex;
use system::*;

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(C)]
pub struct GLDrawElementsIndirectCommand {
    pub nb_indices: GLuint,
    pub nb_instances: GLuint,
    pub first_index: GLuint,
    pub base_vertex: GLuint,
    pub base_instance: GLuint,
}

unsafe fn toast_rendering() {
    use ::std::ptr;

    // Creating the resources
    
    let max_vertices = 0xffffff;
    let max_indices = 0xffffff;
    let max_instances = 0xffffff;

    let mut vao = 0;
    let mut buffers = [0; 6];

    gl::GenVertexArrays(1, &mut vao);
    gl::GenBuffers(buffers.len() as _, buffers.as_mut_ptr());

    let position_vbo = buffers[0];
    let normal_vbo = buffers[1];
    let uv_vbo = buffers[2];
    let model_matrix_vbo = buffers[3];
    let material_index_vbo = buffers[4];
    let ibo = buffers[5];

    let flags = gl::DYNAMIC_STORAGE_BIT;
    gl::NamedBufferStorage(position_vbo, max_vertices * 3 * 4, ptr::null(), flags);
    gl::NamedBufferStorage(normal_vbo, max_vertices * 3 * 4, ptr::null(), flags);
    gl::NamedBufferStorage(uv_vbo, max_vertices * 2 * 4, ptr::null(), flags);
    gl::NamedBufferStorage(model_matrix_vbo, max_instances * 4 * 4 * 4, ptr::null(), flags);
    gl::NamedBufferStorage(material_index_vbo, max_instances * 2, ptr::null(), flags);
    gl::NamedBufferStorage(ibo, max_indices * 4, ptr::null(), flags);

    // Specifying vertex attrib layout

    gl::BindVertexArray(vao);
    gl::EnableVertexAttribArray(VertexAttribIndex::Position as _);
    gl::EnableVertexAttribArray(VertexAttribIndex::Normal as _);
    gl::EnableVertexAttribArray(VertexAttribIndex::UV as _);
    gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 0);
    gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 1);
    gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 2);
    gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 3);
    gl::EnableVertexAttribArray(VertexAttribIndex::MaterialIndex as _);

    gl::VertexAttribDivisor(VertexAttribIndex::Position as _, 0);
    gl::VertexAttribDivisor(VertexAttribIndex::Normal as _, 0);
    gl::VertexAttribDivisor(VertexAttribIndex::UV as _, 0);
    gl::VertexAttribDivisor(VertexAttribIndex::ModelMatrix as GLuint + 0, 1);
    gl::VertexAttribDivisor(VertexAttribIndex::ModelMatrix as GLuint + 1, 1);
    gl::VertexAttribDivisor(VertexAttribIndex::ModelMatrix as GLuint + 2, 1);
    gl::VertexAttribDivisor(VertexAttribIndex::ModelMatrix as GLuint + 3, 1);
    gl::VertexAttribDivisor(VertexAttribIndex::MaterialIndex as _, 1);

    gl::BindBuffer(gl::ARRAY_BUFFER, position_vbo);
    gl::VertexAttribPointer(VertexAttribIndex::Position as _, 3, gl::FLOAT, gl::FALSE, 0, 0 as _);
    gl::BindBuffer(gl::ARRAY_BUFFER, normal_vbo);
    gl::VertexAttribPointer(VertexAttribIndex::Normal as _, 3, gl::FLOAT, gl::FALSE, 0, 0 as _);
    gl::BindBuffer(gl::ARRAY_BUFFER, uv_vbo);
    gl::VertexAttribPointer(VertexAttribIndex::UV as _, 2, gl::FLOAT, gl::FALSE, 0, 0 as _);
    gl::BindBuffer(gl::ARRAY_BUFFER, model_matrix_vbo);
    gl::VertexAttribPointer(VertexAttribIndex::ModelMatrix as GLuint + 0, 4, gl::FLOAT, gl::FALSE, 4*4*4, (0*4*4) as _);
    gl::VertexAttribPointer(VertexAttribIndex::ModelMatrix as GLuint + 1, 4, gl::FLOAT, gl::FALSE, 4*4*4, (1*4*4) as _);
    gl::VertexAttribPointer(VertexAttribIndex::ModelMatrix as GLuint + 2, 4, gl::FLOAT, gl::FALSE, 4*4*4, (2*4*4) as _);
    gl::VertexAttribPointer(VertexAttribIndex::ModelMatrix as GLuint + 3, 4, gl::FLOAT, gl::FALSE, 4*4*4, (3*4*4) as _);
    gl::BindBuffer(gl::ARRAY_BUFFER, material_index_vbo);
    gl::VertexAttribIPointer(VertexAttribIndex::MaterialIndex as _, 1, gl::UNSIGNED_SHORT, 0, 0 as _);
    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    gl::BindVertexArray(0);

    // TODO: Operations:
    // - Add mesh (grab chunks)
    // - Remove mesh (release chunks)
    // - Edit mesh (re-upload data)
    // - Add instance pack
    // - Remove instance pack
    // - Edit instance pack
    //
    // i.e
    // - Allocate N vertices
    // - Allocate N indices
    // - Allocate N instances
    // - Defragment the memory

    // Uploading data

    // gl::NamedBufferSubData(buf, offset, size, data);

    // Drawing

    let m = MemInfo::default();
    let mut cmds = vec![];

    for (i, mesh) in m.instance_ranges.iter().zip(m.instance_range_mesh_entry.iter()) {
        let index_range = &m.index_ranges[*mesh as usize];
        let vertex_range = &m.vertex_ranges[*mesh as usize];
        cmds.push(GLDrawElementsIndirectCommand {
            base_instance: i.start,
            nb_instances: i.end - i.start,
            first_index: index_range.start, // Offset into the index buffer
            nb_indices: index_range.end - index_range.start,
            base_vertex: vertex_range.start, // Value added to indices for vertex retrieval
        });
    }

    gl::BindVertexArray(vao);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
    gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, 0); // read from cpu memory
    gl::MultiDrawElementsIndirect(gx::Topology::Triangles as _, gl::UNSIGNED_INT, cmds.as_ptr() as _, cmds.len() as _, 0);
    gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, 0);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
    gl::BindVertexArray(0);
}

use ::std::ops::Range;
#[derive(Debug, Default, Clone, Hash, PartialEq)]
struct MemInfo {
    // Indexed by mesh
    pub vertex_ranges: Vec<Range<u32>>,
    pub index_ranges: Vec<Range<u32>>,

    // Indexed by instancerange
    pub instance_ranges: Vec<Range<u32>>,
    pub instance_range_mesh_entry: Vec<u32>,
}


#[derive(Debug)]
pub struct GLSystem {
    // Texture arrays
    cubemap_arrays: [GLuint; CubemapArrayID::MAX],
    texture2d_arrays: [GLuint; Texture2DArrayID::MAX],

    // Skybox
    skybox_program: gx::ProgramEx,
    skybox_vbo: gx::Buffer,
    skybox_vao: gx::VertexArray,
}

impl GLSystem {
    pub fn new() -> Self {
        let mut cubemap_arrays = [0; CubemapArrayID::MAX];
        let mut texture2d_arrays = [0; CubemapArrayID::MAX];
        unsafe {
            gl::GenTextures(cubemap_arrays.len() as _, cubemap_arrays.as_mut_ptr());
            gl::GenTextures(texture2d_arrays.len() as _, texture2d_arrays.as_mut_ptr());

            for tex in cubemap_arrays.iter() {
                assert_ne!(*tex, 0);
                gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, *tex);
            }
            gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, 0);

            for tex in texture2d_arrays.iter() {
                assert_ne!(*tex, 0);
                gl::BindTexture(gl::TEXTURE_2D_ARRAY, *tex);
            }
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
        }

        let skybox_vbo = create_skybox_vbo();

        Self {
            cubemap_arrays, texture2d_arrays,
            skybox_program: new_program_ex_unwrap(SKY_VS, SKY_FS),
            skybox_vao: create_skybox_vao(skybox_vbo.gl_id()),
            skybox_vbo,
        }
    }
}

impl Drop for GLSystem {
    fn drop(&mut self) {
        let &mut Self {
            ref mut cubemap_arrays,
            ref mut texture2d_arrays,
            skybox_program: _,
            skybox_vao: _,
            skybox_vbo: _,
        } = self;
        unsafe {
            gl::DeleteTextures(cubemap_arrays.len() as _, cubemap_arrays.as_mut_ptr());
            gl::DeleteTextures(texture2d_arrays.len() as _, texture2d_arrays.as_mut_ptr());
        }
    }
}

impl System for GLSystem {
    fn draw(&mut self, g: &mut G, _d: &Draw) {
        self.process_gpu_cmd_queue(g);

        let Extent2 { w, h } = g.input.canvas_size();
        unsafe {
            gl::Viewport(0, 0, w as _, h as _);
            let Rgba { r, g, b, a } = g.viewport_db().border_color();
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        g.visit_viewports(self);
    }
}

impl GLSystem {
    fn process_gpu_cmd_queue(&mut self, g: &G) {
        for cmd in g.gpu_cmd_queue() {
            self.process_gpu_cmd(g, cmd);
        }
    }
    fn process_gpu_cmd(&mut self, g: &G, cmd: &GpuCmd) {
        unsafe {
            match *cmd {
                GpuCmd::ClearColorEdit => {
                    let Rgba { r, g, b, a } = g.clear_color();
                    gl::ClearColor(r, g, b, a);
                },
                GpuCmd::Texture2DArrayCreate(id) => {
                    let info = g.texture2d_array_info(id).unwrap();
                    gl::TextureStorage3D(self.texture2d_arrays[id.0 as usize], info.levels as _, info.internal_format as _, info.size.w as _, info.size.h as _, info.nb_textures as _);
                },
                GpuCmd::CubemapArrayCreate(id) => {
                    let info = g.cubemap_array_info(id).unwrap();
                    gl::TextureStorage3D(self.cubemap_arrays[id.0 as usize], info.levels as _, info.internal_format as _, info.size.w as _, info.size.h as _, (info.nb_cubemaps * 6) as _);
                },

                GpuCmd::Texture2DArrayDelete(id) => {
                    let tex = &mut self.texture2d_arrays[id.0 as usize];
                    gl::DeleteTextures(1, tex);
                    gl::GenTextures(1, tex);
                    assert_ne!(*tex, 0);
                    gl::BindTexture(gl::TEXTURE_2D_ARRAY, *tex);
                    gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
                },
                GpuCmd::CubemapArrayDelete(id) => {
                    let tex = &mut self.cubemap_arrays[id.0 as usize];
                    gl::DeleteTextures(1, tex);
                    gl::GenTextures(1, tex);
                    assert_ne!(*tex, 0);
                    gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, *tex);
                    gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, 0);
                },


                GpuCmd::Texture2DArrayClear(id, level, color) => {
                    let color: Rgba<f32> = color; // Assert that we're dealing with the correct type
                    gl::ClearTexImage(self.texture2d_arrays[id.0 as usize], level as _, gl::RGBA, gl::FLOAT, color.as_ptr() as _);
                },
                GpuCmd::CubemapArrayClear(id, level, color) => {
                    let color: Rgba<f32> = color; // Assert that we're dealing with the correct type
                    gl::ClearTexImage(self.cubemap_arrays[id.0 as usize], level as _, gl::RGBA, gl::FLOAT, color.as_ptr() as _);
                },

                GpuCmd::Texture2DArraySubImage2D(id, slot, ref img) => {
                    let z = slot;
                    let depth = 1;
                    gl::TextureSubImage3D(self.cubemap_arrays[id.0 as usize], img.level as _, img.x as _, img.y as _, z as _, img.w as _, img.h as _, depth, img.format as _, img.type_ as _, img.data.as_ptr() as _);
                },
                GpuCmd::CubemapArraySubImage2D(id, slot, face, ref img) => {
                    let z = slot * 6 + face as usize;
                    let depth = 1;
                    gl::TextureSubImage3D(self.cubemap_arrays[id.0 as usize], img.level as _, img.x as _, img.y as _, z as _, img.w as _, img.h as _, depth, img.format as _, img.type_ as _, img.data.as_ptr() as _);
                },
            }
        }
    }
}

static SKY_VS: &'static [u8] = b"
#version 450 core

uniform mat4 u_mvp;

layout(location = 0) in vec3 a_position;

out vec3 v_uvw;

void main() {
    gl_Position = (u_mvp * vec4(a_position, 1.0)).xyww; // Z = 1 after perspective divide by w
    v_uvw = a_position;
}
";

static SKY_FS: &'static [u8] = b"
#version 450 core

uniform samplerCubeArray u_cubemap_arrays[32];
uniform uint u_cubemap_array;
uniform float u_cubemap_slot;

in vec3 v_uvw;

out vec4 f_color;

void main() {
    f_color = texture(u_cubemap_arrays[u_cubemap_array], vec4(v_uvw, u_cubemap_slot));
}
";

fn unwrap_or_display_error(r: Result<gx::ProgramEx, String>) -> gx::ProgramEx {
    match r {
        Ok(p) => p,
        Err(e) => {
            error!("GL compile error\n{}", e);
            panic!("GL compile error\n{}", e)
        },
    }
}
fn new_program_ex(vs: &[u8], fs: &[u8]) -> Result<gx::ProgramEx, String> {
    let vs = gx::VertexShader::try_from_source(vs)?;
    let fs = gx::FragmentShader::try_from_source(fs)?;
    let prog = gx::Program::try_from_vert_frag(&vs, &fs)?;
    Ok(gx::ProgramEx::new(prog))
}
fn new_program_ex_unwrap(vs: &[u8], fs: &[u8]) -> gx::ProgramEx {
    unwrap_or_display_error(new_program_ex(vs, fs))
}

use camera::Camera;
use cubemap::CubemapSelector;
use fate::math::Vec4;
use fate::math::Vec3;
use fate::gx::Object;

fn create_skybox_vbo() -> gx::Buffer {
    let vbo = gx::Buffer::new();
    let v = new_skybox_triangle_strip(0.5);
    let flags = 0;
    unsafe {
        gl::NamedBufferStorage(vbo.gl_id(), (v.len() * 3 * 4) as _, v.as_ptr() as _, flags);
    }
    vbo
}

fn create_skybox_vao(vbo: GLuint) -> gx::VertexArray {
    let vao = gx::VertexArray::new();
    unsafe {
        gl::BindVertexArray(vao.gl_id());

        gl::EnableVertexAttribArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, 0 as _);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        gl::BindVertexArray(0);
    }
    vao
}

pub const SKYBOX_NB_VERTICES: usize = 15;

pub fn new_skybox_triangle_strip(s: f32) -> [Vec3<f32>; SKYBOX_NB_VERTICES] {
    [
        Vec3::new(-s,  s,  s), // Degenerate triangle to flip winding. The remainder of the array describes a regular cube strip.

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
    ]
}

impl GLSystem {
    fn draw_skybox(&self, cubemap: CubemapSelector, camera: &Camera) {
        let view = camera.view_matrix();
        let proj = camera.proj_matrix();
        let view_without_translation = {
            let mut r = view;
            r.cols.w = Vec4::unit_w();
            r
        };

        // TODO: glActiveTexture
        unimplemented!();

        unsafe {
            gl::UseProgram(self.skybox_program.inner().gl_id());

            self.skybox_program.set_uniform_primitive("u_proj_matrix", &[proj]);
            self.skybox_program.set_uniform_primitive("u_modelview_matrix", &[view_without_translation]);
            let tabs = self.skybox_program.uniform("u_cubemap_arrays[0]").unwrap(); unimplemented!();
            self.skybox_program.set_uniform_primitive("u_cubemap_array", &[cubemap.array_id.0 as u32]);
            self.skybox_program.set_uniform_primitive("u_cubemap_slot", &[cubemap.cubemap as f32]);

            gl::DepthFunc(gl::LEQUAL);
            gl::BindVertexArray(self.skybox_vao.gl_id());
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, SKYBOX_NB_VERTICES as _);
            gl::BindVertexArray(0);
            gl::DepthFunc(gl::LESS);

            gl::UseProgram(0);
        }
    }
}

impl ViewportVisitor for GLSystem {
    fn accept_leaf_viewport(&mut self, args: AcceptLeafViewport) {
        unsafe {
            let Rect { x, y, w, h } = args.rect;
            gl::Viewport(x as _, y as _, w as _, h as _);

            // Temporary
            gl::Enable(gl::SCISSOR_TEST);

            let (bx, by) = (args.border_px, args.border_px);
            if w < bx+bx || h < by+by {
                return;
            }
            let (x, y, w, h) = (x+bx, y+by, w-bx-bx, h-by-by);
            let Rgba { r, g, b, a } = args.info.clear_color;
            gl::Scissor(x as _, y as _, w as _, h as _);
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT/* | gl::DEPTH_BUFFER_BIT*/);

            self.draw_skybox(args.info.skybox_cubemap_selector, unimplemented!());

            gl::Disable(gl::SCISSOR_TEST);
        }
    }
}
