use v::{Extent2, Rgba};
use gl;
use gl::types::*;
use gx::{self, Object};

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TextureUnit {
    DebugFontAtlas = 1,
    NormalFontAtlas = 2,
}

impl TextureUnit {
    pub fn to_gl(&self) -> GLuint {
        gl::TEXTURE0 + *self as GLuint
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct SamplerParams {
    pub wrap_s: GLenum,
    pub wrap_t: GLenum,
    pub min_filter: GLenum,
    pub mag_filter: GLenum,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Texture2DImage<'a, Pixel: 'a> {
    pub pixels: &'a [Pixel],
    pub size: Extent2<usize>,
    pub mipmap_level: GLint, // 0
    pub internal_format: GLenum,
    pub pixels_format: GLenum,
    pub pixel_element_type: GLenum,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Texture2DInit<'a, Pixel: 'a> {
    pub image: Texture2DImage<'a, Pixel>,
    pub sampler_params: SamplerParams,
    pub do_generate_mipmaps: bool,
}
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Texture2D(pub gx::Texture);

impl SamplerParams {
    pub fn new_clamp_to_edge_linear() -> Self {
        Self {
            wrap_s: gl::CLAMP_TO_EDGE,
            wrap_t: gl::CLAMP_TO_EDGE,
            min_filter: gl::LINEAR,
            mag_filter: gl::LINEAR,
        }
    }
}

impl<'a> Texture2DImage<'a, u8> {
    pub fn from_greyscale_u8(pixels: &'a [u8], size: Extent2<usize>) -> Self {
        Self {
            pixels, size, mipmap_level: 0,
            internal_format: gl::RED,
            pixels_format: gl::RED,
            pixel_element_type: gl::UNSIGNED_BYTE,
        }
    }
}
impl<'a> Texture2DImage<'a, Rgba<u8>> {
    pub fn from_rgba_u8(pixels: &'a [Rgba<u8>], size: Extent2<usize>) -> Self {
        Self {
            pixels, size, mipmap_level: 0,
            internal_format: gl::RGBA,
            pixels_format: gl::RGBA,
            pixel_element_type: gl::UNSIGNED_BYTE,
        }
    }
}

impl Texture2D {
    pub fn new<T>(t: Texture2DInit<T>) -> Self {
        let Texture2DInit { image: img, sampler_params: p, do_generate_mipmaps } = t;
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            assert_ne!(id, 0);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, p.wrap_s as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, p.wrap_t as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, p.min_filter as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, p.mag_filter as _);
            gl::TexImage2D(
                gl::TEXTURE_2D, img.mipmap_level, img.internal_format as _,
                img.size.w as _, img.size.h as _, 0,
                img.pixels_format, img.pixel_element_type, img.pixels.as_ptr() as *const _
            );
            if do_generate_mipmaps {
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }
            Texture2D(gx::Texture::from_gl_id(id))
        }
    }
}


