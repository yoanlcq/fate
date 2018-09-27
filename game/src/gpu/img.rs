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

macro_rules! gpu_texture_internal_format {
    ($($Variant:ident = $val:expr => $size:expr,)+) => {
        #[allow(non_camel_case_types)]
        #[repr(u32)]
        #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
        pub enum GpuTextureInternalFormat {
            $($Variant = $val,)+
        }
        impl GpuTextureInternalFormat {
            pub fn pixel_size(&self) -> Option<usize> {
                self.pixel_bits().map(|x| (x + 7) / 8)
            }
            pub fn pixel_bits(&self) -> Option<usize> {
                match self.pixel_bits_impl() {
                    0 => None,
                    x => Some(x),
                }
            }
            fn pixel_bits_impl(&self) -> usize {
                match *self {
                    $(GpuTextureInternalFormat::$Variant => $size,)+
                }
            }
        }
    };
}

gpu_texture_internal_format!{
     // Base internal formats
     DepthComponent = gl::DEPTH_COMPONENT => 0,
     DepthStencil   = gl::DEPTH_STENCIL => 0,
     StencilIndex   = gl::STENCIL_INDEX => 0,
     R              = gl::RED => 0,
     RG             = gl::RG => 0,
     RGB            = gl::RGB => 0,
     RGBA           = gl::RGBA => 0,
 
     // Sized Internal Formats
     R8             = gl::R8 => 8,
     R8_SNorm       = gl::R8_SNORM => 8,
     R16            = gl::R16 => 16,
     R16_SNorm      = gl::R16_SNORM => 16,
     RG8            = gl::RG8 => 16,
     RG8_SNorm      = gl::RG8_SNORM => 16,
     RG16           = gl::RG16 => 32,
     RG16_SNorm     = gl::RG16_SNORM => 32,
     R3_G3_B2       = gl::R3_G3_B2 => 8,
     RGB4           = gl::RGB4 => 12,
     RGB5           = gl::RGB5 => 15,
     RGB8           = gl::RGB8 => 24,
     RGB8_SNorm     = gl::RGB8_SNORM => 24,
     RGB10          = gl::RGB10 => 30,
     RGB12          = gl::RGB12 => 36,
     RGB16_SNorm    = gl::RGB16_SNORM => 48,
     RGBA2          = gl::RGBA2 => 8,
     RGBA4          = gl::RGBA4 => 16,
     RGB5_A1        = gl::RGB5_A1 => 16,
     RGBA8          = gl::RGBA8 => 32,
     RGBA8_SNorm    = gl::RGBA8_SNORM => 32,
     RGB10_A2       = gl::RGB10_A2 => 32,
     RGB10_A2UI     = gl::RGB10_A2UI => 32,
     RGBA12         = gl::RGBA12 => 4*12,
     RGBA16         = gl::RGBA16 => 4*16,
     SRGB8          = gl::SRGB8 => 24,
     SRGBA8         = gl::SRGB8_ALPHA8 => 32,
     R16F           = gl::R16F => 16,
     RG16F          = gl::RG16F => 32,
     RGB16F         = gl::RGB16F => 48,
     RGBA16F        = gl::RGBA16F => 64,
     R32F           = gl::R32F => 32,
     RG32F          = gl::RG32F => 64,
     RGB32F         = gl::RGB32F => 96,
     RGBA32F        = gl::RGBA32F => 128,
     R11F_G11F_B10F = gl::R11F_G11F_B10F => 32,
     RGB9_E5        = gl::RGB9_E5 => 14,
     R8I            = gl::R8I => 8,
     R8UI           = gl::R8UI => 8,
     R16I           = gl::R16I => 16,
     R16UI          = gl::R16UI => 16,
     R32I           = gl::R32I => 32,
     R32UI          = gl::R32UI => 32,
     RG8I           = gl::RG8I => 16,
     RG8UI          = gl::RG8UI => 16,
     RG16I          = gl::RG16I => 32,
     RG16UI         = gl::RG16UI => 32,
     RG32I          = gl::RG32I => 64,
     RG32UI         = gl::RG32UI => 64,
     RGB8I          = gl::RGB8I => 24,
     RGB8UI         = gl::RGB8UI => 24,
     RGB16I         = gl::RGB16I => 48,
     RGB16UI        = gl::RGB16UI => 48,
     RGB32I         = gl::RGB32I => 96,
     RGB32UI        = gl::RGB32UI => 96,
     RGBA8I         = gl::RGBA8I => 32,
     RGBA8UI        = gl::RGBA8UI => 32,
     RGBA16I        = gl::RGBA16I => 64,
     RGBA16UI       = gl::RGBA16UI => 64,
     RGBA32I        = gl::RGBA32I => 128,
     RGBA32UI       = gl::RGBA32UI => 128,
 
     // Sized Depth and Stencil Internal Formats
     DepthComponent16  = gl::DEPTH_COMPONENT16 => 16,
     DepthComponent24  = gl::DEPTH_COMPONENT24 => 24,
     DepthComponent32  = gl::DEPTH_COMPONENT32 => 32,
     DepthComponent32F = gl::DEPTH_COMPONENT32F => 32,
     Depth24Stencil8   = gl::DEPTH24_STENCIL8 => 32,
     Depth32FStencil8  = gl::DEPTH32F_STENCIL8 => 40,
     StencilIndex8     = gl::STENCIL_INDEX8 => 8,
 
     // Compressed Internal Formats
     CompressedR                      = gl::COMPRESSED_RED => 0,
     CompressedRG                     = gl::COMPRESSED_RG => 0,
     CompressedRGB                    = gl::COMPRESSED_RGB => 0,
     CompressedRGBA                   = gl::COMPRESSED_RGBA => 0,
     CompressedSRGB                   = gl::COMPRESSED_SRGB => 0,
     CompressedSRGBA                  = gl::COMPRESSED_SRGB_ALPHA => 0,
     CompressedRed_RGTC1              = gl::COMPRESSED_RED_RGTC1 => 0,
     CompressedSignedRed_RGTC1        = gl::COMPRESSED_SIGNED_RED_RGTC1 => 0,
     CompressedRG_RGTC2               = gl::COMPRESSED_RG_RGTC2 => 0,
     CompressedSigned_RG_RGTC2        = gl::COMPRESSED_SIGNED_RG_RGTC2 => 0,
     CompressedRGBA_BPTC_UNorm        = gl::COMPRESSED_RGBA_BPTC_UNORM => 0,
     CompressedSRGBA_BPTC_UNorm       = gl::COMPRESSED_SRGB_ALPHA_BPTC_UNORM => 0,
     CompressedRGB_BPTC_SignedFloat   = gl::COMPRESSED_RGB_BPTC_SIGNED_FLOAT => 0,
     CompressedRGB_BPTC_UnsignedFloat = gl::COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT => 0,
 
     // S3TC formats
     CompressedRGB_S3TC_DXT1   = gl::COMPRESSED_RGB_S3TC_DXT1_EXT => 0,
     CompressedSRGB_S3TC_DXT1  = gl::COMPRESSED_SRGB_S3TC_DXT1_EXT => 0,
     CompressedRGBA_S3TC_DXT1  = gl::COMPRESSED_RGBA_S3TC_DXT1_EXT => 0,
     CompressedSRGBA_S3TC_DXT1 = gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT => 0,
     CompressedRGBA_S3TC_DXT3  = gl::COMPRESSED_RGBA_S3TC_DXT3_EXT => 0,
     CompressedSRGBA_S3TC_DXT3 = gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT => 0,
     CompressedRGBA_S3TC_DXT5  = gl::COMPRESSED_RGBA_S3TC_DXT5_EXT => 0,
     CompressedSRGBA_S3TC_DXT5 = gl::COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT => 0,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum GpuTextureFilter {
    Linear = gl::LINEAR,
    Nearest = gl::NEAREST,
    // TODO: Others???
}
