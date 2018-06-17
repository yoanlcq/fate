use gl::types::*;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Attrib {
    pub location: GLuint,
    pub nb_components: GLuint,
    pub gl_type: GLenum,
    pub normalize: bool,
    pub stride: usize,
    pub offset: usize,
}

pub trait ProgramAttribs {
    type Vertex;
    fn attribs(&self) -> Vec<Attrib>;
}

