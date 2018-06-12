// The plan:
//
// Vertex:
// - position 2
// - texcoord 2
//
// Uniforms:
// - mvp
// - color
// - atlas
//
// One character (4 vertices in order):
// - bottom-left;
// - bottom-right;
// - top-right;
// - bottom-left;
//
// glDrawElements(gl::TRIANGLES, count, gl::UNSIGNED_SHORT, 0);


use std::mem;
use gx;
use grx;
use gl::{self, types::*};
use v::{Vec2, Rgba, Mat4};
use font;
use super::{ProgramAttribs, Attrib};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Program {
    program: gx::Program,
    u_mvp: GLint,
    u_atlas: GLint,
    u_color: GLint,
    a_position: GLuint,
    a_texcoords: GLuint,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub position: Vec2<f32>,
    pub texcoords: Vec2<f32>,
}
assert_eq_size!(text_vertex_size; Vertex, [f32; 4]);

impl ProgramAttribs for Program {
    type Vertex = Vertex;
    fn attribs(&self) -> Vec<Attrib> {
        vec![
            Attrib {
                location: self.a_position,
                nb_components: 2,
                gl_type: gl::FLOAT,
                normalize: false,
                stride: mem::size_of::<Vertex>(),
                offset: 0,
            },
            Attrib {
                location: self.a_texcoords,
                nb_components: 2,
                gl_type: gl::FLOAT,
                normalize: false,
                stride: mem::size_of::<Vertex>(),
                offset: mem::size_of::<Vec2<f32>>(),
            },
        ]
    }
}

impl Program {

    const VS: &'static [u8] = b"
#version 130
uniform mat4 u_mvp;
in vec2 a_position;
in vec2 a_texcoords;
out vec2 v_texcoords;
void main() {
    gl_Position = u_mvp * vec4(a_position, 0.0, 1.0);
    v_texcoords = a_texcoords;
}
\0";

    const FS: &'static [u8] = b"
#version 130
uniform sampler2D u_atlas;
uniform vec4 u_color;
in vec2 v_texcoords;
out vec4 f_color;
void main() {
    float alpha = texture2D(u_atlas, v_texcoords).r;
    /*
    if (alpha <= 0.001) {
        discard;
    }
    */
    f_color = vec4(u_color.rgb, alpha);
}
\0";

    pub fn program(&self) -> &gx::Program {
        &self.program
    }
    pub fn new() -> Self {
        let vs = match gx::VertexShader::try_from_source(Self::VS) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to compile vertex shader:\n{}", s);
                panic!("Failed to compile vertex shader:\n{}", s);
            },
        };
        grx::set_label(&vs, b"Text Vertex Shader");
        let fs = match gx::FragmentShader::try_from_source(Self::FS) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to compile fragment shader:\n{}", s);
                panic!("Failed to compile fragment shader:\n{}", s);
            },
        };
        grx::set_label(&vs, b"Text Fragment Shader");
        let program = match gx::Program::try_from_vert_frag(&vs, &fs) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to link GL program:\n{}", s);
                panic!("Failed to link GL program:\n{}", s);
            },
        };
        grx::set_label(&vs, b"Text Program");

        let a_position = program.attrib_location(b"a_position\0").unwrap() as _;
        let a_texcoords = program.attrib_location(b"a_texcoords\0").unwrap() as _;
        let u_mvp = program.uniform_location(b"u_mvp\0").unwrap();
        let u_atlas = program.uniform_location(b"u_atlas\0").unwrap();
        let u_color = program.uniform_location(b"u_color\0").unwrap();

        Self {
            program,
            a_position, a_texcoords,
            u_mvp, u_atlas, u_color,
        }
    }
    pub fn set_uniform_mvp(&self, m: &Mat4<f32>) {
        let transpose = m.gl_should_transpose() as GLboolean;
        unsafe {
            gl::UniformMatrix4fv(self.u_mvp, 1, transpose, m.cols[0].as_ptr());
        }
    }
    pub fn set_uniform_font_atlas_via_font_id(&self, font_id: font::FontID) {
        unsafe {
            gl::Uniform1i(self.u_atlas, grx::TextureUnit::from(font_id) as GLuint as _);
        }
    }
    pub fn set_uniform_color(&self, rgba: Rgba<f32>) {
        unsafe {
            gl::Uniform4fv(self.u_color, 1, rgba.as_ptr());
        }
    }
}

