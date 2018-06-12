use gl;
use gl::types::*;

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextureUnit(u8);

impl TextureUnit {
    #[inline]
    pub fn from_index(i: u8) -> Self {
        TextureUnit(i)
    }
    #[inline]
    pub fn to_glenum(&self) -> GLenum {
        self.0 as GLenum + gl::TEXTURE0
    }
    #[inline]
    pub fn to_index(&self) -> usize {
        self.0 as _
    }
}


