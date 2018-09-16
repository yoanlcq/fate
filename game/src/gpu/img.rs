use fate::gx::gl;
use fate::math::Rgb;
use fate::img;

// TODO: Also move enums to GX

pub fn into_bytes_vec<T>(mut v: Vec<T>) -> Vec<u8> {
    let (ptr, len, cap, sz) = (v.as_mut_ptr(), v.len(), v.capacity(), ::std::mem::size_of::<T>());
    ::std::mem::forget(v);
    unsafe { Vec::from_raw_parts(ptr as *mut u8, len * sz, cap * sz) }
}


// This allows for Vec<u8>, but also Rc-based custom containers for sparing memory.
pub struct CpuPixels(Box<AsRef<[u8]>>);

impl CpuPixels {
    pub fn from_vec<T>(v: Vec<T>) -> Self {
        CpuPixels(Box::new(into_bytes_vec(v)))
    }
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_ref().as_ref()
    }
    pub fn as_ptr(&self) -> *const u8 {
        self.as_slice().as_ptr()
    }
}

use std::fmt::{self, Debug, Formatter};

impl Debug for CpuPixels {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "CpuPixels")
    }
}

impl PartialEq for CpuPixels {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl Eq for CpuPixels {}

impl Clone for CpuPixels {
    fn clone(&self) -> Self {
        Self::from_vec(self.as_slice().to_vec())
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuSubImage3D {
    pub level: u32,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub w: u32,
    pub h: u32,
    pub depth: u32,
    pub format: CpuImgFormat,
    pub type_: CpuImgPixelType,
    pub data: CpuPixels,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuSubImage2D {
    pub level: u32,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub format: CpuImgFormat,
    pub type_: CpuImgPixelType,
    pub data: CpuPixels,
}

impl CpuSubImage2D {
    pub fn from_rgb_u8_pixel(rgb: Rgb<u8>) -> Self {
        CpuSubImage2D {
            level: 0,
            x: 0,
            y: 0,
            w: 1,
            h: 1,
            format: CpuImgFormat::RGB,
            type_: CpuImgPixelType::U8,
            data: CpuPixels::from_vec(vec![rgb]),
        }
    }
    pub fn from_any_image(img: img::AnyImage) -> Self {
        match img {
            img::AnyImage::Rgb8(img) => Self {
                level: 0,
                x: 0,
                y: 0,
                w: img.width() as _,
                h: img.height() as _,
                format: CpuImgFormat::RGB,
                type_: CpuImgPixelType::U8,
                data: CpuPixels::from_vec(img.buf),
            },
            img::AnyImage::Rgba8(img) => Self {
                level: 0,
                x: 0,
                y: 0,
                w: img.width() as _,
                h: img.height() as _,
                format: CpuImgFormat::RGBA,
                type_: CpuImgPixelType::U8,
                data: CpuPixels::from_vec(img.buf),
            },
            img::AnyImage::Gray8(img) => Self {
                level: 0,
                x: 0,
                y: 0,
                w: img.width() as _,
                h: img.height() as _,
                format: CpuImgFormat::R,
                type_: CpuImgPixelType::U8,
                data: CpuPixels::from_vec(img.buf),
            },
            img::AnyImage::GrayAlpha8(img) => Self {
                level: 0,
                x: 0,
                y: 0,
                w: img.width() as _,
                h: img.height() as _,
                format: CpuImgFormat::RG,
                type_: CpuImgPixelType::U8,
                data: CpuPixels::from_vec(img.buf),
            },
        }
    }
}


#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum CpuImgFormat {
    // Depth and stencil
    DepthComponent = gl::DEPTH_COMPONENT,
    StencilIndex = gl::STENCIL_INDEX,
    DepthStencil = gl::DEPTH_STENCIL,
    // For transfers of normalized integer or floating-point color image data
    R = gl::RED,
    G = gl::GREEN,
    B = gl::BLUE,
    RG = gl::RG,
    RGB = gl::RGB,
    RGBA = gl::RGBA,
    BGR = gl::BGR,
    BGRA = gl::BGRA,
    // For transfers of non-normalized integer data
    R_Integer = gl::RED_INTEGER,
    G_Integer = gl::GREEN_INTEGER,
    B_Integer = gl::BLUE_INTEGER,
    RG_Integer = gl::RG_INTEGER,
    RGB_Integer = gl::RGB_INTEGER,
    BGR_Integer = gl::BGR_INTEGER,
    RGBA_Integer = gl::RGBA_INTEGER,
    BGRA_Integer = gl::BGRA_INTEGER,
}

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum CpuImgPixelType {
    U8 = gl::UNSIGNED_BYTE,
    I8 = gl::BYTE,
    U16 = gl::UNSIGNED_SHORT,
    I16 = gl::SHORT,
    U32 = gl::UNSIGNED_INT,
    I32 = gl::INT,
    F32 = gl::FLOAT,
    U8_3_3_2 = gl::UNSIGNED_BYTE_3_3_2,
    U8_2_3_3_Rev = gl::UNSIGNED_BYTE_2_3_3_REV,
    U16_5_6_5 = gl::UNSIGNED_SHORT_5_6_5,
    U16_5_6_5_Rev = gl::UNSIGNED_SHORT_5_6_5_REV,
    U16_4_4_4_4 = gl::UNSIGNED_SHORT_4_4_4_4,
    U16_4_4_4_4_Rev = gl::UNSIGNED_SHORT_4_4_4_4_REV,
    U16_5_5_5_1 = gl::UNSIGNED_SHORT_5_5_5_1,
    U16_1_5_5_5_Rev = gl::UNSIGNED_SHORT_1_5_5_5_REV,
    U32_8_8_8_8 = gl::UNSIGNED_INT_8_8_8_8,
    U32_8_8_8_8_Rev = gl::UNSIGNED_INT_8_8_8_8_REV,
    U32_10_10_10_2 = gl::UNSIGNED_INT_10_10_10_2,
    U32_2_10_10_10_Rev = gl::UNSIGNED_INT_2_10_10_10_REV,
}

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum GpuTextureInternalFormat {
     // Base internal formats
     DepthComponent = gl::DEPTH_COMPONENT,
     DepthStencil   = gl::DEPTH_STENCIL,
     StencilIndex   = gl::STENCIL_INDEX,
     R              = gl::RED,
     RG             = gl::RG,
     RGB            = gl::RGB,
     RGBA           = gl::RGBA,
 
     // Sized Internal Formats
     R8             = gl::R8,
     R8_SNorm       = gl::R8_SNORM,
     R16            = gl::R16,
     R16_SNorm      = gl::R16_SNORM,
     RG8            = gl::RG8,
     RG8_SNorm      = gl::RG8_SNORM,
     RG16           = gl::RG16,
     RG16_SNorm     = gl::RG16_SNORM,
     R3_G3_B2       = gl::R3_G3_B2,
     RGB4           = gl::RGB4,
     RGB5           = gl::RGB5,
     RGB8           = gl::RGB8,
     RGB8_SNorm     = gl::RGB8_SNORM,
     RGB10          = gl::RGB10,
     RGB12          = gl::RGB12,
     RGB16_SNorm    = gl::RGB16_SNORM,
     RGBA2          = gl::RGBA2,
     RGBA4          = gl::RGBA4,
     RGB5_A1        = gl::RGB5_A1,
     RGBA8          = gl::RGBA8,
     RGBA8_SNorm    = gl::RGBA8_SNORM,
     RGB10_A2       = gl::RGB10_A2,
     RGB10_A2UI     = gl::RGB10_A2UI,
     RGBA12         = gl::RGBA12,
     RGBA16         = gl::RGBA16,
     SRGB8          = gl::SRGB8,
     SRGBA8         = gl::SRGB8_ALPHA8,
     R16F           = gl::R16F,
     RG16F          = gl::RG16F,
     RGB16F         = gl::RGB16F,
     RGBA16F        = gl::RGBA16F,
     R32F           = gl::R32F,
     RG32F          = gl::RG32F,
     RGB32F         = gl::RGB32F,
     RGBA32F        = gl::RGBA32F,
     R11F_G11F_B10F = gl::R11F_G11F_B10F,
     RGB9_E5        = gl::RGB9_E5,
     R8I            = gl::R8I,
     R8UI           = gl::R8UI,
     R16I           = gl::R16I,
     R16UI          = gl::R16UI,
     R32I           = gl::R32I,
     R32UI          = gl::R32UI,
     RG8I           = gl::RG8I,
     RG8UI          = gl::RG8UI,
     RG16I          = gl::RG16I,
     RG16UI         = gl::RG16UI,
     RG32I          = gl::RG32I,
     RG32UI         = gl::RG32UI,
     RGB8I          = gl::RGB8I,
     RGB8UI         = gl::RGB8UI,
     RGB16I         = gl::RGB16I,
     RGB16UI        = gl::RGB16UI,
     RGB32I         = gl::RGB32I,
     RGB32UI        = gl::RGB32UI,
     RGBA8I         = gl::RGBA8I,
     RGBA8UI        = gl::RGBA8UI,
     RGBA16I        = gl::RGBA16I,
     RGBA16UI       = gl::RGBA16UI,
     RGBA32I        = gl::RGBA32I,
     RGBA32UI       = gl::RGBA32UI,
 
     // Sized Depth and Stencil Internal Formats
     DepthComponent16  = gl::DEPTH_COMPONENT16,
     DepthComponent24  = gl::DEPTH_COMPONENT24,
     DepthComponent32  = gl::DEPTH_COMPONENT32,
     DepthComponent32F = gl::DEPTH_COMPONENT32F,
     Depth24Stencil8   = gl::DEPTH24_STENCIL8,
     Depth32FStencil8  = gl::DEPTH32F_STENCIL8,
     StencilIndex8     = gl::STENCIL_INDEX8,
 
     // Compressed Internal Formats
     CompressedR                      = gl::COMPRESSED_RED,
     CompressedRG                     = gl::COMPRESSED_RG,
     CompressedRGB                    = gl::COMPRESSED_RGB,
     CompressedRGBA                   = gl::COMPRESSED_RGBA,
     CompressedSRGB                   = gl::COMPRESSED_SRGB,
     CompressedSRGBA                  = gl::COMPRESSED_SRGB_ALPHA,
     CompressedRed_RGTC1              = gl::COMPRESSED_RED_RGTC1,
     CompressedSignedRed_RGTC1        = gl::COMPRESSED_SIGNED_RED_RGTC1,
     CompressedRG_RGTC2               = gl::COMPRESSED_RG_RGTC2,
     CompressedSigned_RG_RGTC2        = gl::COMPRESSED_SIGNED_RG_RGTC2,
     CompressedRGBA_BPTC_UNorm        = gl::COMPRESSED_RGBA_BPTC_UNORM,
     CompressedSRGBA_BPTC_UNorm       = gl::COMPRESSED_SRGB_ALPHA_BPTC_UNORM,
     CompressedRGB_BPTC_SignedFloat   = gl::COMPRESSED_RGB_BPTC_SIGNED_FLOAT,
     CompressedRGB_BPTC_UnsignedFloat = gl::COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT,
 
     // S3TC formats
     CompressedRGB_S3TC_DXT1   = gl::COMPRESSED_RGB_S3TC_DXT1_EXT,
     CompressedSRGB_S3TC_DXT1  = gl::COMPRESSED_SRGB_S3TC_DXT1_EXT,
     CompressedRGBA_S3TC_DXT1  = gl::COMPRESSED_RGBA_S3TC_DXT1_EXT,
     CompressedSRGBA_S3TC_DXT1 = gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT,
     CompressedRGBA_S3TC_DXT3  = gl::COMPRESSED_RGBA_S3TC_DXT3_EXT,
     CompressedSRGBA_S3TC_DXT3 = gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT,
     CompressedRGBA_S3TC_DXT5  = gl::COMPRESSED_RGBA_S3TC_DXT5_EXT,
     CompressedSRGBA_S3TC_DXT5 = gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum GpuTextureFilter {
    Linear = gl::LINEAR,
    Nearest = gl::NEAREST,
    // TODO: Others???
}
