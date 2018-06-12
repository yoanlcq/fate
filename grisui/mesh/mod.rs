use gl::types::*;

pub mod vertex_array;
pub mod index_array;
pub mod color_mesh;
pub mod text;


pub trait ProgramAttribs {
    type Vertex;
    fn attribs(&self) -> Vec<Attrib>;
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Attrib {
    pub location: GLuint,
    pub nb_components: GLuint,
    pub gl_type: GLenum,
    pub normalize: bool,
    pub stride: usize,
    pub offset: usize,
}

