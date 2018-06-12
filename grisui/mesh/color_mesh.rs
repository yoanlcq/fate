use std::mem;
use gx;
use grx;
use gl::{self, types::*};
use v::{Vec3, Rgba, Mat4};
use super::{ProgramAttribs, Attrib};


#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Program {
    program: gx::Program,
    u_mvp: GLint,
    u_is_drawing_points: GLint,
    a_position: GLuint,
    a_color: GLuint,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub position: Vec3<f32>,
    pub color: Rgba<f32>,
}
assert_eq_size!(vertex_size; Vertex, [f32; 7]);

impl ProgramAttribs for Program {
    type Vertex = Vertex;
    fn attribs(&self) -> Vec<Attrib> {
        vec![
            Attrib {
                location: self.a_position,
                nb_components: 3,
                gl_type: gl::FLOAT,
                normalize: false,
                stride: mem::size_of::<Vertex>(),
                offset: 0
            },
            Attrib {
                location: self.a_color,
                nb_components: 4,
                gl_type: gl::FLOAT,
                normalize: false,
                stride: mem::size_of::<Vertex>(),
                offset: mem::size_of::<Vec3<f32>>(),
            },
        ]
    }
}


impl Program {

    const VS: &'static [u8] = b"
#version 130
uniform mat4 u_mvp;
in vec3 a_position;
in vec4 a_color;
out vec4 v_color;
void main() {
    gl_Position = u_mvp * vec4(a_position, 1.0);
    v_color = a_color;
}
\0";


    const FS: &'static [u8] = b"
#version 130
uniform bool u_is_drawing_points;
in vec4 v_color;
out vec4 f_color;
void main() {
    if(u_is_drawing_points) {
        vec2 from_center = gl_PointCoord - vec2(0.5f);
        float d = length(from_center);
        if(d > 0.5f)
            discard;
    }
    f_color = v_color;
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
        grx::set_label(&vs, b"Mesh Vertex Shader");
        let fs = match gx::FragmentShader::try_from_source(Self::FS) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to compile fragment shader:\n{}", s);
                panic!("Failed to compile fragment shader:\n{}", s);
            },
        };
        grx::set_label(&fs, b"Mesh Fragment Shader");
        let program = match gx::Program::try_from_vert_frag(&vs, &fs) {
            Ok(i) => i,
            Err(s) => {
                error!("Failed to link GL program:\n{}", s);
                panic!("Failed to link GL program:\n{}", s);
            },
        };
        grx::set_label(&program, b"Color Mesh Program");

        let a_position = program.attrib_location(b"a_position\0").unwrap() as _;
        let a_color = program.attrib_location(b"a_color\0").unwrap() as _;
        let u_mvp = program.uniform_location(b"u_mvp\0").unwrap();
        let u_is_drawing_points = program.uniform_location(b"u_is_drawing_points\0").unwrap();

        Self {
            program, u_mvp, u_is_drawing_points, a_position, a_color,
        }
    }
    pub fn set_uniform_mvp(&self, m: &Mat4<f32>) {
        let transpose = m.gl_should_transpose() as GLboolean;
        unsafe {
            gl::UniformMatrix4fv(self.u_mvp, 1, transpose, m.cols[0].as_ptr());
        }
    }
    pub fn set_uniform_is_drawing_points(&self, yes: bool) {
        unsafe {
            gl::Uniform1i(self.u_is_drawing_points, yes as _);
        }
    }
}

