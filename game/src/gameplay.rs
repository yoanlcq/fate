use fate::math::{Rgb, Rgba};
use viewport::ViewportNode;
use eid::EID;
use cubemap::{CubemapSelector, CubemapArrayID, CubemapArrayInfo, CubemapFace};
use texture2d::{Texture2DArrayID, Texture2DArrayInfo};
use gpu::{GpuTextureInternalFormat, CpuSubImage2D, CpuImgFormat, CpuImgPixelType, CpuPixels, GpuTextureFilter};
use system::*;

mod cubemap {
    use super::*;
    pub const RGB8_1L_1x1: CubemapArrayID = CubemapArrayID(0);
    pub const RGB8_1L_1024x1024: CubemapArrayID = CubemapArrayID(1);
}

mod texture2d {
    use super::*;
    pub const RGB8_1L_1x1: Texture2DArrayID = Texture2DArrayID(0);
    pub const RGB8_1L_1024x1024: Texture2DArrayID = Texture2DArrayID(1);
}


#[derive(Debug)]
pub struct Gameplay;

impl Gameplay {
    pub fn new(g: &mut G) -> Self {
        g.cubemap_array_create(cubemap::RGB8_1L_1x1, CubemapArrayInfo {
            nb_levels: 1,
            internal_format: GpuTextureInternalFormat::RGB8,
            size: Extent2::new(1, 1),
            nb_cubemaps: 16,
        });
        g.cubemap_array_create(cubemap::RGB8_1L_1024x1024, CubemapArrayInfo {
            nb_levels: 1,
            internal_format: GpuTextureInternalFormat::RGB8,
            size: Extent2::broadcast(1024),
            nb_cubemaps: 6,
        });

        g.texture2d_array_create(texture2d::RGB8_1L_1x1, Texture2DArrayInfo {
            nb_levels: 1,
            internal_format: GpuTextureInternalFormat::RGB8,
            size: Extent2::broadcast(1),
            nb_slots: 2,
        });
        g.texture2d_array_create(texture2d::RGB8_1L_1024x1024, Texture2DArrayInfo {
            nb_levels: 1,
            internal_format: GpuTextureInternalFormat::RGB8,
            size: Extent2::broadcast(1024),
            nb_slots: 2,
        });

        fn pixel(rgb: Rgb<u8>) -> CpuSubImage2D {
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

        // TODO:
        // GL_TEXTURE_MAX_ANISOTROPY GL_MAX_TEXTURE_MAX_ANISOTROPY GL_LINEAR_MIPMAP_LINEAR
        // ARB_texture_filter_anisotropic EXT_texture_filter_anisotropic
        g.cubemap_array_clear(cubemap::RGB8_1L_1x1, 0, Rgba::magenta());

        g.cubemap_array_set_min_filter(cubemap::RGB8_1L_1x1, GpuTextureFilter::Nearest);
        g.cubemap_array_set_mag_filter(cubemap::RGB8_1L_1x1, GpuTextureFilter::Nearest);

        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 0, CubemapFace::PositiveX, pixel(Rgb::new(000, 000, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 0, CubemapFace::NegativeX, pixel(Rgb::new(000, 000, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 0, CubemapFace::PositiveY, pixel(Rgb::new(000, 000, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 0, CubemapFace::NegativeY, pixel(Rgb::new(000, 000, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 0, CubemapFace::PositiveZ, pixel(Rgb::new(000, 000, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 0, CubemapFace::NegativeZ, pixel(Rgb::new(000, 000, 000)));

        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 1, CubemapFace::PositiveX, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 1, CubemapFace::NegativeX, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 1, CubemapFace::PositiveY, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 1, CubemapFace::NegativeY, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 1, CubemapFace::PositiveZ, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 1, CubemapFace::NegativeZ, pixel(Rgb::new(255, 255, 255)));

        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 2, CubemapFace::PositiveX, pixel(Rgb::new(255, 000, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 2, CubemapFace::NegativeX, pixel(Rgb::new(000, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 2, CubemapFace::PositiveY, pixel(Rgb::new(000, 255, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 2, CubemapFace::NegativeY, pixel(Rgb::new(255, 000, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 2, CubemapFace::PositiveZ, pixel(Rgb::new(000, 000, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 2, CubemapFace::NegativeZ, pixel(Rgb::new(255, 255, 000)));

        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 3, CubemapFace::PositiveX, pixel(Rgb::new(000, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 3, CubemapFace::NegativeX, pixel(Rgb::new(000, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 3, CubemapFace::PositiveY, pixel(Rgb::new(000, 000, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 3, CubemapFace::NegativeY, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 3, CubemapFace::PositiveZ, pixel(Rgb::new(000, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 3, CubemapFace::NegativeZ, pixel(Rgb::new(000, 255, 255)));

        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 4, CubemapFace::PositiveX, pixel(Rgb::new(255, 175,  45)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 4, CubemapFace::NegativeX, pixel(Rgb::new(255, 175,  45)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 4, CubemapFace::PositiveY, pixel(Rgb::new(255, 000, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 4, CubemapFace::NegativeY, pixel(Rgb::new(255, 255, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 4, CubemapFace::PositiveZ, pixel(Rgb::new(255, 175,  45)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 4, CubemapFace::NegativeZ, pixel(Rgb::new(255, 175,  45)));

        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 5, CubemapFace::PositiveX, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 5, CubemapFace::NegativeX, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 5, CubemapFace::PositiveY, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 5, CubemapFace::NegativeY, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 5, CubemapFace::PositiveZ, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1x1, 5, CubemapFace::NegativeZ, pixel(Rgb::new(255, 255, 255)));

        g.cubemap_array_clear(cubemap::RGB8_1L_1024x1024, 0, Rgba::magenta());

        g.cubemap_array_set_min_filter(cubemap::RGB8_1L_1024x1024, GpuTextureFilter::Linear);
        g.cubemap_array_set_mag_filter(cubemap::RGB8_1L_1024x1024, GpuTextureFilter::Linear);


        g.texture2d_array_clear(texture2d::RGB8_1L_1x1, 0, Rgba::magenta());

        g.texture2d_array_set_min_filter(texture2d::RGB8_1L_1x1, GpuTextureFilter::Nearest);
        g.texture2d_array_set_mag_filter(texture2d::RGB8_1L_1x1, GpuTextureFilter::Nearest);

        g.texture2d_array_sub_image_2d(texture2d::RGB8_1L_1x1, 0, pixel(Rgb::new(000, 000, 000)));
        g.texture2d_array_sub_image_2d(texture2d::RGB8_1L_1x1, 1, pixel(Rgb::new(255, 255, 255)));

        g.texture2d_array_clear(texture2d::RGB8_1L_1024x1024, 0, Rgba::magenta());

        g.texture2d_array_set_min_filter(texture2d::RGB8_1L_1024x1024, GpuTextureFilter::Linear);
        g.texture2d_array_set_mag_filter(texture2d::RGB8_1L_1024x1024, GpuTextureFilter::Linear);


        // TODO: Upload cubemap textures
        // TODO: Upload font atlas
        Gameplay
    }
}

impl System for Gameplay {
}
