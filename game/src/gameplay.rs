use std::path::PathBuf;
use std::io;
use fate::math::{Rgb, Rgba};
use fate::mt;
use fate::img;
use viewport::ViewportNode;
use eid::EID;
use cubemap::{CubemapSelector, CubemapArrayID, CubemapArrayInfo, CubemapFace};
use texture2d::{Texture2DArrayID, Texture2DArrayInfo};
use gpu::{GpuTextureInternalFormat, CpuSubImage2D, CpuImgFormat, CpuImgPixelType, CpuPixels, GpuTextureFilter};
use system::*;

mod cubemap {
    use super::*;
    pub const RGB8_1L_1X1: CubemapArrayID = CubemapArrayID(0);
    pub const RGB8_1L_1024X1024: CubemapArrayID = CubemapArrayID(1);
}

mod texture2d {
    use super::*;
    pub const RGB8_1L_1X1: Texture2DArrayID = Texture2DArrayID(0);
    pub const RGB8_1L_1024X1024: Texture2DArrayID = Texture2DArrayID(1);
}


type ImgFuture = mt::Future<mt::Then<mt::ReadFile, mt::Async<io::Result<img::Result<(img::Metadata, img::AnyImage)>>>>>;

#[derive(Debug)]
struct CubemapFaceRequest {
    future: Option<ImgFuture>,
    path: PathBuf,
    cubemap_index: u32,
    face: CubemapFace,
}


#[derive(Debug)]
pub struct Gameplay {
    cubemap_face_requests: Vec<CubemapFaceRequest>,
}

impl Gameplay {
    pub fn new(g: &mut G) -> Self {
        {
            let mut leaf = g.viewport_db_mut().root_node().value.unwrap_leaf().borrow_mut();
            leaf.skybox_cubemap_selector = Some(CubemapSelector { array_id: cubemap::RGB8_1L_1024X1024, cubemap: 0, });
        }

        g.cubemap_array_create(cubemap::RGB8_1L_1X1, CubemapArrayInfo {
            nb_levels: 1,
            internal_format: GpuTextureInternalFormat::RGB8,
            size: Extent2::new(1, 1),
            nb_cubemaps: 16,
        });
        g.cubemap_array_create(cubemap::RGB8_1L_1024X1024, CubemapArrayInfo {
            nb_levels: 1,
            internal_format: GpuTextureInternalFormat::RGB8,
            size: Extent2::broadcast(1024),
            nb_cubemaps: 6,
        });

        g.texture2d_array_create(texture2d::RGB8_1L_1X1, Texture2DArrayInfo {
            nb_levels: 1,
            internal_format: GpuTextureInternalFormat::RGB8,
            size: Extent2::broadcast(1),
            nb_slots: 2,
        });
        g.texture2d_array_create(texture2d::RGB8_1L_1024X1024, Texture2DArrayInfo {
            nb_levels: 1,
            internal_format: GpuTextureInternalFormat::RGB8,
            size: Extent2::broadcast(1024),
            nb_slots: 2,
        });

        fn pixel(rgb: Rgb<u8>) -> CpuSubImage2D {
            CpuSubImage2D::from_rgb_u8_pixel(rgb)
        }

        // TODO:
        // GL_TEXTURE_MAX_ANISOTROPY GL_MAX_TEXTURE_MAX_ANISOTROPY GL_LINEAR_MIPMAP_LINEAR
        // ARB_texture_filter_anisotropic EXT_texture_filter_anisotropic
        g.cubemap_array_clear(cubemap::RGB8_1L_1X1, 0, Rgba::magenta());

        g.cubemap_array_set_min_filter(cubemap::RGB8_1L_1X1, GpuTextureFilter::Nearest);
        g.cubemap_array_set_mag_filter(cubemap::RGB8_1L_1X1, GpuTextureFilter::Nearest);

        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 0, CubemapFace::PositiveX, pixel(Rgb::new(000, 000, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 0, CubemapFace::NegativeX, pixel(Rgb::new(000, 000, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 0, CubemapFace::PositiveY, pixel(Rgb::new(000, 000, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 0, CubemapFace::NegativeY, pixel(Rgb::new(000, 000, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 0, CubemapFace::PositiveZ, pixel(Rgb::new(000, 000, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 0, CubemapFace::NegativeZ, pixel(Rgb::new(000, 000, 000)));

        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 1, CubemapFace::PositiveX, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 1, CubemapFace::NegativeX, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 1, CubemapFace::PositiveY, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 1, CubemapFace::NegativeY, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 1, CubemapFace::PositiveZ, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 1, CubemapFace::NegativeZ, pixel(Rgb::new(255, 255, 255)));

        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 2, CubemapFace::PositiveX, pixel(Rgb::new(255, 000, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 2, CubemapFace::NegativeX, pixel(Rgb::new(000, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 2, CubemapFace::PositiveY, pixel(Rgb::new(000, 255, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 2, CubemapFace::NegativeY, pixel(Rgb::new(255, 000, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 2, CubemapFace::PositiveZ, pixel(Rgb::new(000, 000, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 2, CubemapFace::NegativeZ, pixel(Rgb::new(255, 255, 000)));

        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 3, CubemapFace::PositiveX, pixel(Rgb::new(000, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 3, CubemapFace::NegativeX, pixel(Rgb::new(000, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 3, CubemapFace::PositiveY, pixel(Rgb::new(000, 000, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 3, CubemapFace::NegativeY, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 3, CubemapFace::PositiveZ, pixel(Rgb::new(000, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 3, CubemapFace::NegativeZ, pixel(Rgb::new(000, 255, 255)));

        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 4, CubemapFace::PositiveX, pixel(Rgb::new(255, 175,  45)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 4, CubemapFace::NegativeX, pixel(Rgb::new(255, 175,  45)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 4, CubemapFace::PositiveY, pixel(Rgb::new(255, 000, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 4, CubemapFace::NegativeY, pixel(Rgb::new(255, 255, 000)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 4, CubemapFace::PositiveZ, pixel(Rgb::new(255, 175,  45)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 4, CubemapFace::NegativeZ, pixel(Rgb::new(255, 175,  45)));

        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 5, CubemapFace::PositiveX, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 5, CubemapFace::NegativeX, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 5, CubemapFace::PositiveY, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 5, CubemapFace::NegativeY, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 5, CubemapFace::PositiveZ, pixel(Rgb::new(255, 255, 255)));
        g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1X1, 5, CubemapFace::NegativeZ, pixel(Rgb::new(255, 255, 255)));

        g.cubemap_array_clear(cubemap::RGB8_1L_1024X1024, 0, Rgba::magenta());

        g.cubemap_array_set_min_filter(cubemap::RGB8_1L_1024X1024, GpuTextureFilter::Linear);
        g.cubemap_array_set_mag_filter(cubemap::RGB8_1L_1024X1024, GpuTextureFilter::Linear);


        g.texture2d_array_clear(texture2d::RGB8_1L_1X1, 0, Rgba::magenta());

        g.texture2d_array_set_min_filter(texture2d::RGB8_1L_1X1, GpuTextureFilter::Nearest);
        g.texture2d_array_set_mag_filter(texture2d::RGB8_1L_1X1, GpuTextureFilter::Nearest);

        g.texture2d_array_sub_image_2d(texture2d::RGB8_1L_1X1, 0, pixel(Rgb::new(000, 000, 000)));
        g.texture2d_array_sub_image_2d(texture2d::RGB8_1L_1X1, 1, pixel(Rgb::new(255, 255, 255)));

        g.texture2d_array_clear(texture2d::RGB8_1L_1024X1024, 0, Rgba::magenta());

        g.texture2d_array_set_min_filter(texture2d::RGB8_1L_1024X1024, GpuTextureFilter::Linear);
        g.texture2d_array_set_mag_filter(texture2d::RGB8_1L_1024X1024, GpuTextureFilter::Linear);


        // Upload cubemap textures (async)
        
        let dir = g.res.data_path().join(PathBuf::from("art/3rdparty/mayhem"));
        let suffixes = CubemapFace::TERRAGEN_SUFFIXES;
        let extension = "jpg";
        let mut cubemap_face_requests = vec![];
        for (cubemap_index, name) in ["grouse", "aqua4", "h2s", "flame"].iter().enumerate() {
            for suffix in suffixes.iter() {
                cubemap_face_requests.push(CubemapFaceRequest {
                    path: dir.join(format!("{}_{}.{}", name, suffix, extension)),
                    cubemap_index: cubemap_index as _,
                    face: CubemapFace::try_from_terragen_suffix(suffix).unwrap(),
                    future: None,
                });
            }
        }

        for req in cubemap_face_requests.iter_mut() {
            use self::mt::TaskExt;
            let future = g.mt.schedule(mt::ReadFile::new(&req.path).then(|result: io::Result<Vec<u8>>| {
                mt::Async::new(move || result.map(|data| img::load_from_memory(data)))
            }));
            req.future = Some(future);
        }

        // TODO: Upload font atlas
        
        Gameplay {
            cubemap_face_requests,
        }
    }
}

impl Gameplay {
    fn pump_cubemap_faces(&mut self, g: &mut G) {
        loop {
            let mut complete = None;

            for (i, req) in self.cubemap_face_requests.iter().enumerate() {
                let future = req.future.as_ref().unwrap();
                if future.is_complete() {
                    complete = Some(i);
                    break;
                }

                let _progress = match future.poll() {
                    mt::Either::Left(fp) => format!("{}%", if fp.nsize == 0 { 0. } else { fp.nread as f32 / fp.nsize as f32 }),
                    mt::Either::Right(_) => format!("Converting..."),
                };
                // text += &format!("Loading {} (z = {}): {}\n", future.as_ref().first().path().display(), z, progress);
            }

            match complete {
                None => break,
                Some(i) => {
                    let mut req = self.cubemap_face_requests.remove(i);
                    match req.future.take().unwrap().wait() {
                        Ok(Ok((_, img))) => {
                            g.cubemap_array_sub_image_2d(cubemap::RGB8_1L_1024X1024, req.cubemap_index as _, req.face, CpuSubImage2D::from_any_image(img));
                            info!("Loaded `{}`", req.path.display());
                        },
                        _ => unimplemented!{},
                    }
                }
            }
        }
    }
}

impl System for Gameplay {
    fn draw(&mut self, g: &mut G, _: &Draw) {
        self.pump_cubemap_faces(g);
    }
}
