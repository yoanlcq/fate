use fate::math::{Extent2, Rgba, Rect};
use fate::gx::{self, gl::{self, types::*}};

use gpu::GpuCmd;
use viewport::{ViewportVisitor, AcceptLeafViewport};
use cubemap::{CubemapArrayID};
use texture2d::Texture2DArrayID;
use mesh::VertexAttribIndex;
use system::*;

pub struct GLSystem {
    // Texture arrays
    cubemap_arrays: [GLuint; CubemapArrayID::MAX],
    texture2d_arrays: [GLuint; Texture2DArrayID::MAX],

    // Skybox
    skybox_program: gx::ProgramEx,
    #[allow(dead_code)]
    skybox_vbo: gx::Buffer,
    skybox_vao: gx::VertexArray,
}

impl GLSystem {
    pub fn new() -> Self {
        let mut cubemap_arrays = [0; CubemapArrayID::MAX];
        let mut texture2d_arrays = [0; Texture2DArrayID::MAX];
        unsafe {
            gl::CreateTextures(gl::TEXTURE_CUBE_MAP_ARRAY, cubemap_arrays.len() as _, cubemap_arrays.as_mut_ptr());
            gl::CreateTextures(gl::TEXTURE_2D_ARRAY, texture2d_arrays.len() as _, texture2d_arrays.as_mut_ptr());
        }

        let skybox_vbo = create_skybox_vbo();

        Self {
            cubemap_arrays,
            texture2d_arrays,
            skybox_program: new_program_ex_unwrap(SKY_VS, SKY_FS),
            skybox_vao: create_skybox_vao(skybox_vbo.gl_id()),
            skybox_vbo,
        }
    }
    pub fn cubemap_array(&self, id: CubemapArrayID) -> GLuint { self.cubemap_arrays[id.0 as usize] }
    pub fn texture2d_array(&self, id: Texture2DArrayID) -> GLuint { self.texture2d_arrays[id.0 as usize] }
    pub fn cubemap_array_mut(&mut self, id: CubemapArrayID) -> &mut GLuint { &mut self.cubemap_arrays[id.0 as usize] }
    pub fn texture2d_array_mut(&mut self, id: Texture2DArrayID) -> &mut GLuint { &mut self.texture2d_arrays[id.0 as usize] }
}

impl Drop for GLSystem {
    fn drop(&mut self) {
        let &mut Self {
            ref mut cubemap_arrays,
            ref mut texture2d_arrays,
            ..
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

        g.visit_viewports(&mut GLViewportVisitor { g, sys: self, });
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
                    gl::TextureStorage3D(self.texture2d_array(id), info.nb_levels as _, info.internal_format as _, info.size.w as _, info.size.h as _, info.nb_slots as _);
                },
                GpuCmd::CubemapArrayCreate(id) => {
                    let info = g.cubemap_array_info(id).unwrap();
                    gl::TextureStorage3D(self.cubemap_array(id), info.nb_levels as _, info.internal_format as _, info.size.w as _, info.size.h as _, (info.nb_cubemaps * 6) as _);
                },

                GpuCmd::Texture2DArrayDelete(id) => {
                    let tex = self.texture2d_array_mut(id);
                    gl::DeleteTextures(1, tex);
                    gl::CreateTextures(gl::TEXTURE_2D_ARRAY, 1, tex);
                },
                GpuCmd::CubemapArrayDelete(id) => {
                    let tex = self.cubemap_array_mut(id);
                    gl::DeleteTextures(1, tex);
                    gl::CreateTextures(gl::TEXTURE_CUBE_MAP_ARRAY, 1, tex);
                },


                GpuCmd::Texture2DArrayClear(id, level, color) => {
                    let color: Rgba<f32> = color; // Assert that we're dealing with the correct type
                    gl::ClearTexImage(self.texture2d_array(id), level as _, gl::RGBA, gl::FLOAT, color.as_ptr() as _);
                },
                GpuCmd::CubemapArrayClear(id, level, color) => {
                    let color: Rgba<f32> = color; // Assert that we're dealing with the correct type
                    gl::ClearTexImage(self.cubemap_array(id), level as _, gl::RGBA, gl::FLOAT, color.as_ptr() as _);
                },

                GpuCmd::Texture2DArraySubImage2D(id, slot, ref img) => {
                    let z = slot;
                    let depth = 1;
                    gl::TextureSubImage3D(self.texture2d_array(id), img.level as _, img.x as _, img.y as _, z as _, img.w as _, img.h as _, depth, img.format as _, img.type_ as _, img.data.as_ptr() as _);
                },
                GpuCmd::CubemapArraySubImage2D(id, slot, face, ref img) => {
                    let z = slot * 6 + face as usize;
                    let depth = 1;
                    gl::TextureSubImage3D(self.cubemap_array(id), img.level as _, img.x as _, img.y as _, z as _, img.w as _, img.h as _, depth, img.format as _, img.type_ as _, img.data.as_ptr() as _);
                },

                GpuCmd::CubemapArraySetMinFilter(id, filter)   => gl::TextureParameteri(self.cubemap_array(id), gl::TEXTURE_MIN_FILTER, filter as _),
                GpuCmd::CubemapArraySetMagFilter(id, filter)   => gl::TextureParameteri(self.cubemap_array(id), gl::TEXTURE_MAG_FILTER, filter as _),
                GpuCmd::Texture2DArraySetMinFilter(id, filter) => gl::TextureParameteri(self.texture2d_array(id), gl::TEXTURE_MIN_FILTER, filter as _),
                GpuCmd::Texture2DArraySetMagFilter(id, filter) => gl::TextureParameteri(self.texture2d_array(id), gl::TEXTURE_MAG_FILTER, filter as _),
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

uniform samplerCubeArray u_cubemap_array;
uniform float u_cubemap_slot;

in vec3 v_uvw;

out vec4 f_color;

void main() {
    f_color = texture(u_cubemap_array, vec4(v_uvw, u_cubemap_slot));
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
    let v = new_skybox_triangle_strip(0.5);
    let flags = 0;
    unsafe {
        let mut vbo = 0;
        gl::CreateBuffers(1, &mut vbo);
        gl::NamedBufferStorage(vbo, (v.len() * 3 * 4) as _, v.as_ptr() as _, flags);
        gx::Buffer::from_gl_id(vbo)
    }
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

use camera::View;

impl GLSystem {
    fn draw_skybox(&self, cubemap: CubemapSelector, camera: &View) {
        let mvp = {
            let view = camera.view_matrix();
            let proj = camera.proj_matrix();
            let view_without_translation = {
                let mut r = view;
                r.cols.w = Vec4::unit_w();
                r
            };
            proj * view_without_translation
        };

        unsafe {
            gl::UseProgram(self.skybox_program.inner().gl_id());

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, self.cubemap_array(cubemap.array_id));

            self.skybox_program.set_uniform_primitive("u_mvp", &[mvp]);
            self.skybox_program.set_uniform("u_cubemap_array", gx::GLSLType::SamplerCubeMapArray, &[0_i32]);
            self.skybox_program.set_uniform_primitive("u_cubemap_slot", &[cubemap.cubemap as f32]);

            gl::DepthFunc(gl::LEQUAL);
            gl::BindVertexArray(self.skybox_vao.gl_id());
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, SKYBOX_NB_VERTICES as _);
            gl::BindVertexArray(0);
            gl::DepthFunc(gl::LESS);

            gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, 0);

            gl::UseProgram(0);
        }
    }
}

struct GLViewportVisitor<'a> {
    pub g: &'a G,
    pub sys: &'a GLSystem,
}

impl<'a> ViewportVisitor for GLViewportVisitor<'a> {
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

            if let Some(skybox_cubemap_selector) = args.info.skybox_cubemap_selector {
                let eid = args.info.camera;
                let view = View {
                    xform: *self.g.eid_xform(eid).unwrap(),
                    camera: *self.g.eid_camera(eid).unwrap(),
                    viewport: Rect { x, y, w, h },
                };
                self.sys.draw_skybox(skybox_cubemap_selector, &view);
            }

            gl::Disable(gl::SCISSOR_TEST);
        }
    }
}
