use fate::math::{Vec3, Vec4};
use fate::gx::{self, Object, gl::{self, types::*}};

use camera::Camera;
use camera::View;
use cubemap::CubemapSelector;

#[derive(Debug, PartialEq, Eq)]
pub struct GLSkybox {
    program: gx::ProgramEx,
    #[allow(dead_code)]
    vbo: gx::Buffer,
    vao: gx::VertexArray,
}

impl GLSkybox {
    pub fn new() -> Self {
        let vbo = create_skybox_vbo();
        Self {
            program: super::new_program_ex_unwrap(SKY_VS, SKY_FS),
            vao: create_skybox_vao(vbo.gl_id()),
            vbo,
        }
    }
}

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

impl GLSkybox {
    pub fn draw(&self, cubemap: CubemapSelector, cubemap_array_tex: GLuint, camera: &View) {
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
            gl::UseProgram(self.program.inner().gl_id());

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, cubemap_array_tex);

            self.program.set_uniform_primitive("u_mvp", &[mvp]);
            self.program.set_uniform("u_cubemap_array", gx::GLSLType::SamplerCubeMapArray, &[0_i32]);
            self.program.set_uniform_primitive("u_cubemap_slot", &[cubemap.cubemap as f32]);

            gl::DepthFunc(gl::LEQUAL);
            gl::BindVertexArray(self.vao.gl_id());
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, SKYBOX_NB_VERTICES as _);
            gl::BindVertexArray(0);
            gl::DepthFunc(gl::LESS);

            gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, 0);

            gl::UseProgram(0);
        }
    }
}
