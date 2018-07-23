extern crate imgref;
extern crate image;
extern crate stb_image;
extern crate fate_math as math;

pub use imgref::*;
pub use image::{
    ImageResult as Result,
    ImageError as Error,
    // traits
    Pixel,
};

use std::fs;
use std::io;
use std::path::Path;
use math::Extent2;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ChannelDataType {
    UnsignedBits,
    FloatingBits,
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct ChannelInfo {
    data_type: Option<ChannelDataType>,
    bits: u32,
}

impl ChannelInfo {
    pub fn bits(&self) -> u32 { self.bits }
    pub fn data_type(&self) -> Option<ChannelDataType> { self.data_type }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum PixelSemantic {
    Rgb,
    Rgba,
    Gray,
    GrayAlpha,
    Palette,
    // Exotic formats
    Rgbe8,
    // 10_10_2....
}

impl PixelSemantic {
    pub fn nb_channels(&self) -> usize {
        match *self {
            PixelSemantic::Rgb => 3,
            PixelSemantic::Rgba => 4,
            PixelSemantic::Gray => 1,
            PixelSemantic::GrayAlpha => 2,
            PixelSemantic::Palette => 1,
            PixelSemantic::Rgbe8 => 4,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct PixelFormat {
    semantic: PixelSemantic,
    channels: [ChannelInfo; 4], // find nb_channels from semantic
}

impl PixelFormat {
    pub fn semantic(&self) -> PixelSemantic { self.semantic }
    pub fn channels(&self) -> &[ChannelInfo] { &self.channels[..self.semantic.nb_channels()] }
    pub fn bits(&self) -> u32 { self.channels().iter().map(|c| c.bits()).sum() }

    fn new_unchecked(semantic: PixelSemantic, bits_per_pixel: u32, t: ChannelDataType) -> Self {
        Self { semantic, channels: [ChannelInfo { data_type: Some(t), bits: bits_per_pixel / semantic.nb_channels() as u32 }; 4] }
    }
    fn from_colortype_and_uniform_channel_datatype(c: image::ColorType, t: ChannelDataType) -> Self {
        match c {
            image::ColorType::Gray(bpp) => Self::new_unchecked(PixelSemantic::Gray, bpp as _, t),
            image::ColorType::RGB(bpp) => Self::new_unchecked(PixelSemantic::Rgb, bpp as _, t),
            image::ColorType::Palette(bpp) => Self::new_unchecked(PixelSemantic::Palette, bpp as _, t),
            image::ColorType::GrayA(bpp) => Self::new_unchecked(PixelSemantic::GrayAlpha, bpp as _, t),
            image::ColorType::RGBA(bpp) => Self::new_unchecked(PixelSemantic::Rgba, bpp as _, t),
        }
    }
    fn colortype(&self) -> image::ColorType {
        let bpp = self.bits() as u8;
        match self.semantic() {
            PixelSemantic::Gray => image::ColorType::Gray(bpp),
            PixelSemantic::Rgb => image::ColorType::RGB(bpp),
            PixelSemantic::Palette => image::ColorType::Palette(bpp),
            PixelSemantic::GrayAlpha => image::ColorType::GrayA(bpp),
            PixelSemantic::Rgba | PixelSemantic::Rgbe8 => image::ColorType::RGBA(bpp),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ImageFormat {
    // Supported by the image crate
    PNG,
    JPEG,
    GIF,
    WEBP,
    PNM,
    TIFF,
    TGA,
    BMP,
    ICO,
    HDR,
    DXT,
    // Supported by stb_image (amongst some of the others above, obviously)
    PIC,
    PSD,
}

impl ImageFormat {
    fn from_image_crate_format(f: image::ImageFormat) -> Self {
        match f {
            image::ImageFormat::PNG  => ImageFormat::PNG,
            image::ImageFormat::JPEG => ImageFormat::JPEG,
            image::ImageFormat::GIF  => ImageFormat::GIF,
            image::ImageFormat::WEBP => ImageFormat::WEBP,
            image::ImageFormat::PNM  => ImageFormat::PNM,
            image::ImageFormat::TIFF => ImageFormat::TIFF,
            image::ImageFormat::TGA  => ImageFormat::TGA,
            image::ImageFormat::BMP  => ImageFormat::BMP,
            image::ImageFormat::ICO  => ImageFormat::ICO,
            image::ImageFormat::HDR  => ImageFormat::HDR,
        }
    }
    fn to_image_crate_format(&self) -> Option<image::ImageFormat> {
        match *self {
            ImageFormat::PNG  => Some(image::ImageFormat::PNG),
            ImageFormat::JPEG => Some(image::ImageFormat::JPEG),
            ImageFormat::GIF  => Some(image::ImageFormat::GIF),
            ImageFormat::WEBP => Some(image::ImageFormat::WEBP),
            ImageFormat::PNM  => Some(image::ImageFormat::PNM),
            ImageFormat::TIFF => Some(image::ImageFormat::TIFF),
            ImageFormat::TGA  => Some(image::ImageFormat::TGA),
            ImageFormat::BMP  => Some(image::ImageFormat::BMP),
            ImageFormat::ICO  => Some(image::ImageFormat::ICO),
            ImageFormat::HDR  => Some(image::ImageFormat::HDR),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Metadata {
    pub image_format: ImageFormat,
    pub pixel_format: PixelFormat,
    pub size: Extent2<u32>,
}

pub fn format<R: io::BufRead + io::Seek>(mut r: R) -> Result<ImageFormat> {
    // image::guess_format() actually only compares the start of the file to some pre-defined magic headers, for all formats!
    let mut magic = [0_u8; 16];
    let magic = {
        let start = r.seek(io::SeekFrom::Current(0)).map_err(Error::IoError)?;
        let magic_len = r.read(&mut magic).map_err(Error::IoError)?;
        r.seek(io::SeekFrom::Start(start)).map_err(Error::IoError)?;
        &magic[..magic_len]
    };
    Ok(ImageFormat::from_image_crate_format(image::guess_format(magic)?))
}

pub fn metadata<R: io::BufRead + io::Seek>(mut r: R) -> Result<Metadata> {
    let image_format = format(&mut r)?;
    match image_format {
        ImageFormat::PNG  => decoder_metadata(image_format, image::png::PNGDecoder::new(r)),
        ImageFormat::JPEG => decoder_metadata(image_format, image::jpeg::JPEGDecoder::new(r)),
        ImageFormat::GIF  => decoder_metadata(image_format, image::gif::Decoder::new(r)),
        ImageFormat::WEBP => decoder_metadata(image_format, image::webp::WebpDecoder::new(r)),
        ImageFormat::PNM  => decoder_metadata(image_format, image::pnm::PNMDecoder::new(r)?),
        ImageFormat::TIFF => decoder_metadata(image_format, image::tiff::TIFFDecoder::new(r)?),
        ImageFormat::TGA  => decoder_metadata(image_format, image::tga::TGADecoder::new(r)),
        ImageFormat::BMP  => decoder_metadata(image_format, image::bmp::BMPDecoder::new(r)),
        ImageFormat::ICO  => decoder_metadata(image_format, image::ico::ICODecoder::new(r)?),
        ImageFormat::HDR  => decoder_metadata(image_format, image::hdr::HDRAdapter::new(r)?),
        ImageFormat::DXT  => Err(Error::UnsupportedError(format!("TODO: DXT loader needs to know width, height, and DXTVariant ahead of time"))),
        ImageFormat::PIC  => Err(Error::UnsupportedError(format!("TODO: use stb_image"))),
        ImageFormat::PSD  => Err(Error::UnsupportedError(format!("TODO: use stb_image"))),
    }
}

fn decoder_metadata<T: image::ImageDecoder>(image_format: ImageFormat, mut decoder: T) -> Result<Metadata> {
    Ok(Metadata {
        image_format,
        pixel_format: PixelFormat::from_colortype_and_uniform_channel_datatype(decoder.colortype()?, ChannelDataType::UnsignedBits),
        size: decoder.dimensions()?.into(),
    })
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HDRDecoder {
    strict: bool,
}

impl HDRDecoder {
    pub fn with_strictness(strict: bool) -> Self {
        Self { strict }
    }
    pub fn read_native_rgbe8<R: io::BufRead>(&self, r: R) -> Result<ImgVec<image::hdr::RGBE8Pixel>> {
        let (d, m) = self.decoder_and_metadata(r)?;
        Ok(ImgVec::new(d.read_image_native()?, m.width as _, m.height as _))
    }
    pub fn read_hdr_rgb_f32<R: io::BufRead>(&self, r: R) -> Result<ImgVec<image::Rgb<f32>>> {
        let (d, m) = self.decoder_and_metadata(r)?;
        Ok(ImgVec::new(d.read_image_hdr()?, m.width as _, m.height as _))
    }
    pub fn read_ldr_rgb_u8<R: io::BufRead>(&self, r: R) -> Result<ImgVec<image::Rgb<u8>>> {
        let (d, m) = self.decoder_and_metadata(r)?;
        Ok(ImgVec::new(d.read_image_ldr()?, m.width as _, m.height as _))
    }
    fn decoder_and_metadata<R: io::BufRead>(&self, r: R) -> Result<(image::hdr::HDRDecoder<R>, image::hdr::HDRMetadata)> {
        let d = image::hdr::HDRDecoder::with_strictness(r, self.strict)?;
        let m = d.metadata();
        Ok((d, m))
    }
}

pub fn write_hdr_rgb_f32<W: io::Write>(w: W, img: ImgRef<image::Rgb<f32>>) -> io::Result<()> {
    image::hdr::HDREncoder::new(w).encode(img.buf, img.width(), img.height())
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AnyImage {
    Gray8(ImgVec<image::Luma<u8>>),
    GrayAlpha8(ImgVec<image::LumaA<u8>>),
    Rgb8(ImgVec<image::Rgb<u8>>),
    Rgba8(ImgVec<image::Rgba<u8>>),
}


fn imgvec_from_imagebuffer<P: image::Pixel + 'static>(buf: image::ImageBuffer<P, Vec<P::Subpixel>>) -> ImgVec<P> {
    let (w, h) = buf.dimensions();
    ImgVec::new(unsafe { ::std::mem::transmute(buf.into_raw()) }, w as _, h as _)
}

impl AnyImage {
    fn from_dynamic(d: image::DynamicImage) -> Self {
        match d {
            image::DynamicImage::ImageLuma8 (buf) => AnyImage::Gray8(imgvec_from_imagebuffer(buf)),
            image::DynamicImage::ImageLumaA8(buf) => AnyImage::GrayAlpha8(imgvec_from_imagebuffer(buf)),
            image::DynamicImage::ImageRgb8  (buf) => AnyImage::Rgb8(imgvec_from_imagebuffer(buf)),
            image::DynamicImage::ImageRgba8 (buf) => AnyImage::Rgba8(imgvec_from_imagebuffer(buf)),
        }
    }
}

pub fn read<R: io::BufRead + io::Seek>(mut r: R) -> Result<(Metadata, AnyImage)> {
    let m = metadata(&mut r)?;
    Ok((m, read_with_format(&mut r, m.image_format)?))
}
pub fn read_with_format<R: io::BufRead + io::Seek>(r: R, format: ImageFormat) -> Result<AnyImage> {
    match format {
        ImageFormat::PNG  |
        ImageFormat::JPEG |
        ImageFormat::GIF  |
        ImageFormat::WEBP |
        ImageFormat::PNM  |
        ImageFormat::TIFF |
        ImageFormat::TGA  |
        ImageFormat::BMP  |
        ImageFormat::ICO  |
        ImageFormat::HDR  => image::load(r, format.to_image_crate_format().unwrap()).map(AnyImage::from_dynamic),
        ImageFormat::DXT  => Err(Error::UnsupportedError(format!("TODO: DXTDecoder needs to know width, height, and DXTVariant ahead of time"))),
        ImageFormat::PIC  => Err(Error::UnsupportedError(format!("TODO: PIC: use stb_image"))),
        ImageFormat::PSD  => Err(Error::UnsupportedError(format!("TODO: PSD: use stb_image"))),
    }
}

pub fn load<P: AsRef<Path>>(path: P) -> Result<(Metadata, AnyImage)> {
    read(io::BufReader::new(fs::File::open(path).map_err(Error::IoError)?))
}
pub fn load_from_memory(mem: Vec<u8>) -> Result<(Metadata, AnyImage)> {
    read(io::Cursor::new(mem))
}
pub fn save<P: AsRef<Path>>(path: P, metadata: Metadata, pixels: &[u8]) -> Result<()> {
    write(fs::File::create(path).map_err(Error::IoError)?, metadata, pixels)
}

pub fn write<W: io::Write>(mut out: W, metadata: Metadata, pixels: &[u8]) -> Result<()> {
    let Extent2 { w, h } = metadata.size;
    let colortype = metadata.pixel_format.colortype();
    match metadata.image_format {
        ImageFormat::PNG  => image::png::PNGEncoder::new(out).encode(pixels, w, h, colortype).map_err(Error::IoError),
        ImageFormat::JPEG => image::jpeg::JPEGEncoder::new(&mut out).encode(pixels, w, h, colortype).map_err(Error::IoError),
        ImageFormat::GIF  => Err(Error::UnsupportedError(format!("TODO: Make a separate function to export GIFs"))),
        ImageFormat::WEBP => Err(Error::UnsupportedError(format!("WEBP encoding is not supported yet"))),
        ImageFormat::PNM  => image::pnm::PNMEncoder::new(out).encode(pixels, w, h, colortype).map_err(Error::IoError),
        ImageFormat::TIFF => Err(Error::UnsupportedError(format!("TIFF encoding is not supported yet"))),
        ImageFormat::TGA  => Err(Error::UnsupportedError(format!("TGA encoding is not supported yet"))),
        ImageFormat::BMP  => image::bmp::BMPEncoder::new(&mut out).encode(pixels, w, h, colortype).map_err(Error::IoError),
        ImageFormat::ICO  => image::ico::ICOEncoder::new(out).encode(pixels, w, h, colortype).map_err(Error::IoError),
        ImageFormat::HDR  => Err(Error::UnsupportedError(format!("Use HDR::write_hdr_rgb_f32() instead"))),
        ImageFormat::DXT  => Err(Error::UnsupportedError(format!("TODO: DXT encoder needs to know DXTVariant"))),
        ImageFormat::PIC  => Err(Error::UnsupportedError(format!("PIC encoding is not supported"))),
        ImageFormat::PSD  => Err(Error::UnsupportedError(format!("PSD encoding is not supported"))),
    }
}