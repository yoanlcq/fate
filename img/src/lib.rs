extern crate imgref;
// BMP, DXT, GIF, HDR, ICO, JPG, PNG, PNM, TGA, TIFF, WEBP
extern crate image;
// PIC, PSD
extern crate stb_image;
extern crate vek;

pub use imgref::*;
pub use image::{
    ImageResult as Result,
    ImageFormat,
    DynamicImage,
    ColorType,
    DecodingResult,
    FilterType,
    ImageError as Error,
    ImageOutputFormat as OutputFormat,
    // traits
    ConvertBuffer,
    GenericImage,
    ImageDecoder,
    Pixel,
    Primitive,
};

use std::fs;
use std::io;
use std::path::Path;

/*
pub fn load<P: AsRef<Path>>(path: P) -> Result<DynamicImage>  {
    image::open(path)
}
pub fn file_format<P: AsRef<Path>>(path: P) -> Result<Format> {
    format(&fs::read(path).map_err(Error::IoError)?)
}

// FIXME: DynamicImage = Imgvec
pub fn read<R: io::BufRead + io::Seek>(r: R, format: Format) -> Result<DynamicImage> {
    image::load(r, format)
}
// FIXME: Take an imgref instead
pub fn save<P: AsRef<Path>>(path: P, buf: &[u8], width: u32, height: u32, color: ColorType) -> io::Result<()>  {
    image::save_buffer(path, buf, width, height, color)
}
pub fn format(buffer: &[u8]) -> Result<ImageFormat> {
    image::guess_format(buffer)
}
pub fn from_memory_with_format(buf: &[u8], format: ImageFormat) -> Result<Image> {
    image::load_from_memory_with_format(buf, format)
}
*/

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    format: ImageFormat,
    dynamic: DynamicImage,
}

pub fn from_memory(buffer: Vec<u8>) -> Result<Image> {
    image::load_from_memory(buffer)
}

pub mod hdr {
    use super::*;
    // TODO: with sctrictness
    pub fn read_rgbe8<R: io::BufRead>(r: R) -> Result<ImgVec<RGBE8Pixel>> {
        let dec = image::hdr::HDRDecoder::new(r)?;
        dec.metadata();
        dec.read_image_native()
    }
    pub fn read_rgb_f32<R: io::BufRead>(r: R) -> Result<ImgVec<vek::Rgb<f32>>> {
        let dec = image::hdr::HDRDecoder::new(r)?;
        dec.metadata();
        dec.read_image_hdr()
    }
    pub fn read_rgb_u8<R: io::BufRead>(r: R) -> Result<ImgVec<vek::Rgb<u8>>> {
        let dec = image::hdr::HDRDecoder::new(r)?;
        dec.metadata();
        dec.read_image_ldr()
    }
}