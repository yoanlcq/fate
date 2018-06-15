use std::rc::Rc;
use std::ffi::CStr;
use std::mem;
use std::ptr;
use std::slice;
use std::os::raw::*;
use vek::{Vec2, Extent2};
use imgref::{ImgVec, ImgRef};
use freetype_sys::{self, *};

macro_rules! ft_error_codes {
    ($($variant:ident)+) => {
        #[repr(i32)]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum Error {
            $(#[allow(warnings)] $variant = freetype_sys::$variant),+
        }
        impl Error {
            pub fn try_from_i32(i: i32) -> Option<Self> {
                match i {
                    $(freetype_sys::$variant => Some(Error::$variant),)+
                    _ => None,
                }
            }
        }
    }
}
ft_error_codes!{
    FT_Err_Array_Too_Large	
    FT_Err_Bad_Argument	
    FT_Err_Bbx_Too_Big	
    FT_Err_CMap_Table_Missing	
    FT_Err_Cannot_Open_Resource	
    FT_Err_Cannot_Open_Stream	
    FT_Err_Cannot_Render_Glyph	
    FT_Err_Code_Overflow	
    FT_Err_Corrupted_Font_Glyphs	
    FT_Err_Corrupted_Font_Header	
    FT_Err_Could_Not_Find_Context	
    FT_Err_Debug_OpCode	
    FT_Err_Divide_By_Zero	
    FT_Err_ENDF_In_Exec_Stream	
    FT_Err_Execution_Too_Long	
    FT_Err_Hmtx_Table_Missing	
    FT_Err_Horiz_Header_Missing	
    FT_Err_Ignore	
    FT_Err_Invalid_Argument	
    FT_Err_Invalid_Cache_Handle	
    FT_Err_Invalid_CharMap_Format	
    FT_Err_Invalid_CharMap_Handle	
    FT_Err_Invalid_Character_Code	
    FT_Err_Invalid_CodeRange	
    FT_Err_Invalid_Composite	
    FT_Err_Invalid_Driver_Handle	
    FT_Err_Invalid_Face_Handle	
    FT_Err_Invalid_File_Format	
    FT_Err_Invalid_Frame_Operation	
    FT_Err_Invalid_Frame_Read	
    FT_Err_Invalid_Glyph_Format	
    FT_Err_Invalid_Glyph_Index	
    FT_Err_Invalid_Handle	
    FT_Err_Invalid_Horiz_Metrics	
    FT_Err_Invalid_Library_Handle	
    FT_Err_Invalid_Offset	
    FT_Err_Invalid_Opcode	
    FT_Err_Invalid_Outline	
    FT_Err_Invalid_PPem	
    FT_Err_Invalid_Pixel_Size	
    FT_Err_Invalid_Post_Table	
    FT_Err_Invalid_Post_Table_Format	
    FT_Err_Invalid_Reference	
    FT_Err_Invalid_Size_Handle	
    FT_Err_Invalid_Slot_Handle	
    FT_Err_Invalid_Stream_Handle	
    FT_Err_Invalid_Stream_Operation	
    FT_Err_Invalid_Stream_Read	
    FT_Err_Invalid_Stream_Seek	
    FT_Err_Invalid_Stream_Skip	
    FT_Err_Invalid_Table	
    FT_Err_Invalid_Version	
    FT_Err_Invalid_Vert_Metrics	
    FT_Err_Locations_Missing	
    FT_Err_Lower_Module_Version	
    FT_Err_Max	
    FT_Err_Missing_Bbx_Field	
    FT_Err_Missing_Chars_Field	
    FT_Err_Missing_Encoding_Field	
    FT_Err_Missing_Font_Field	
    FT_Err_Missing_Fontboundingbox_Field	
    FT_Err_Missing_Module	
    FT_Err_Missing_Property	
    FT_Err_Missing_Size_Field	
    FT_Err_Missing_Startchar_Field	
    FT_Err_Missing_Startfont_Field	
    FT_Err_Name_Table_Missing	
    FT_Err_Nested_DEFS	
    FT_Err_Nested_Frame_Access	
    FT_Err_No_Unicode_Glyph_Name	
    // FT_Err_Ok	
    FT_Err_Out_Of_Memory	
    FT_Err_Post_Table_Missing	
    FT_Err_Raster_Corrupted	
    FT_Err_Raster_Negative_Height	
    FT_Err_Raster_Overflow	
    FT_Err_Raster_Uninitialized	
    FT_Err_Stack_Overflow	
    FT_Err_Stack_Underflow	
    FT_Err_Syntax_Error	
    FT_Err_Table_Missing	
    FT_Err_Too_Few_Arguments	
    FT_Err_Too_Many_Caches	
    FT_Err_Too_Many_Drivers	
    FT_Err_Too_Many_Extensions	
    FT_Err_Too_Many_Function_Defs	
    FT_Err_Too_Many_Hints	
    FT_Err_Too_Many_Instruction_Defs	
    FT_Err_Unimplemented_Feature	
    FT_Err_Unknown_File_Format	
    FT_Err_Unlisted_Object
}

fn ft_result(status: i32) -> Result<(), Error> {
    if status == FT_Err_Ok {
        Ok(())
    } else {
        Err(Error::try_from_i32(status).unwrap_or(Error::FT_Err_Max))
    }
}

fn i32_from_26_6(ft_pos: FT_Pos) -> i32 {
    (ft_pos >> 6) as _
}
fn f64_from_26_6(ft_pos: FT_Pos) -> f64 {
    let dec = i32_from_26_6(ft_pos);
    let frac = ft_pos & 0b111111;
    dec as f64 + (frac as f64 / 0b111111 as f64)
}
fn f64_from_16_16(i: FT_Pos) -> f64 {
    let dec = i >> 16;
    let frac = i & 0xffff;
    dec as f64 + (frac as f64 / 0xffff as f64)
}
fn f64_to_26_6(x: f64) -> FT_Pos {
    let dec = x as i32;
    let frac = ((x - dec as f64) * 0b111111 as f64) as i32;
    (dec << 6 | frac) as _
}


#[derive(Debug)]
struct FreeType {
    ft_library: FT_Library,
}

impl Drop for FreeType {
    fn drop(&mut self) {
        unsafe {
            FT_Done_FreeType(self.ft_library);
        }
    }
}

#[derive(Debug)]
pub struct FontLoader {
    ft: Rc<FreeType>,
}

impl FontLoader {
    pub fn new() -> Result<Self, Error> {
        let mut ft_library: FT_Library = unsafe { mem::uninitialized() };
        ft_result(unsafe { FT_Init_FreeType(&mut ft_library) })?;
        Ok(Self { ft: Rc::new(FreeType { ft_library })})
    }

    pub fn create_font(&self, mem: &[u8], desired_height: u32) -> Result<Font, Error> {
        unsafe {
            let mut ft_face: FT_Face = mem::uninitialized();
            ft_result(FT_New_Memory_Face(self.ft.ft_library, mem.as_ptr(), mem.len() as _, 0, &mut ft_face))?;
            let (w, h) = (0, desired_height);
            ft_result(FT_Set_Pixel_Sizes(ft_face, w, h))?;
            let font = Font {
                ft: Rc::clone(&self.ft),
                ft_face,
            };
            Ok(font)
        }
    }
}

#[derive(Debug)]
pub struct Font {
    ft: Rc<FreeType>,
    ft_face: FT_Face,
}

impl Drop for Font {
    fn drop(&mut self) {
        unsafe {
            FT_Done_Face(self.ft_face);
        }
    }
}

impl Font {
    fn ft_face(&self) -> &FT_FaceRec {
        unsafe { &*self.ft_face }
    }
    fn ft_face_flags(&self) -> c_long {
        self.ft_face().face_flags
    }
    fn ft_size_metrics(&self) -> &FT_Size_Metrics {
        unsafe { &(*self.ft_face().size).metrics }
    }
    // These are not mutually exclusive
    pub fn is_outline_font(&self) -> bool {
        (self.ft_face_flags() & FT_FACE_FLAG_SCALABLE) != 0
    }
    pub fn is_bitmap_font(&self) -> bool {
        (self.ft_face_flags() & FT_FACE_FLAG_FIXED_SIZES) != 0
    }
    pub fn is_fixed_width(&self) -> bool {
        (self.ft_face_flags() & FT_FACE_FLAG_FIXED_WIDTH) != 0
    }
    pub fn has_kerning(&self) -> bool {
        (self.ft_face_flags() & FT_FACE_FLAG_KERNING) != 0
    }
    pub fn family_name(&self) -> String {
        let ascii = self.ft_face().family_name;
        assert!(!ascii.is_null());
        unsafe {
            CStr::from_ptr(ascii).to_string_lossy().into()
        }
    }
    pub fn style_name(&self) -> Option<String> {
        let ascii = self.ft_face().style_name;
        if ascii.is_null() {
            None
        } else {
            unsafe {
                Some(CStr::from_ptr(ascii).to_string_lossy().into())
            }
        }
    }
    pub fn pixels_per_em(&self) -> Extent2<u16> {
        let m = self.ft_size_metrics();
        Extent2::new(m.x_ppem, m.y_ppem)
    }
    // The values below are 26.6, but already rounded to integer values as the FreeType2 docs claim.
    pub fn ascender_px(&self) -> i32 {
        i32_from_26_6(self.ft_size_metrics().ascender)
    }
    pub fn descender_px(&self) -> i32 {
        i32_from_26_6(self.ft_size_metrics().descender)
    }
    pub fn height_px(&self) -> u32 {
        let h = i32_from_26_6(self.ft_size_metrics().height);
        assert!(h >= 0);
        h as u32
    }
    pub fn max_horizontal_advance_px(&self) -> i32 {
        i32_from_26_6(self.ft_size_metrics().max_advance)
    }
    pub fn glyph(&self, c: char) -> GlyphLoader {
        GlyphLoader {
            font: self,
            c,
            pedantic: false,
            render_u8_monochrome_bitmap: false,
        }
    }
}


#[derive(Debug)]
pub struct GlyphLoader<'a> {
    font: &'a Font,
    c: char,
    pedantic: bool,
    render_u8_monochrome_bitmap: bool,
}

impl<'a> GlyphLoader<'a> {
    pub fn pedantic(mut self) -> Self { self.pedantic = true; self }
    pub fn render_u8_monochrome_bitmap(mut self) -> Self { self.render_u8_monochrome_bitmap = true; self }
    pub fn load(self) -> Result<Glyph, Error> { 
        let Self {
            font, c, pedantic, render_u8_monochrome_bitmap,
        } = self;

        let mut flags = FT_LOAD_DEFAULT;
        if pedantic {
            flags |= FT_LOAD_PEDANTIC;
        }
        ft_result(unsafe { FT_Load_Char(font.ft_face, c as u64, flags) })?;
        let mut ft_glyph_slot = unsafe { ptr::read(&*(*font.ft_face).glyph) };

        let u8_monochrome_bitmap = if !render_u8_monochrome_bitmap {
            None
        } else {
            // FT_RENDER_MODE_MONO   => 1-bit bitmaps
            // FT_RENDER_MODE_NORMAL => 8-bit anti-aliased
            // FT_RENDER_MODE_LCD    => Horizontal RGB and BGR
            // FT_RENDER_MODE_LCD_V  => Vertical RGB and BGR
            ft_result(unsafe { FT_Render_Glyph(&mut ft_glyph_slot, FT_RENDER_MODE_NORMAL) })?;
            let bmp = &ft_glyph_slot.bitmap;
            let buf = unsafe {
                slice::from_raw_parts(bmp.buffer, (bmp.rows*bmp.pitch) as usize)
            };
            Some(ImgVec::new_stride(buf.to_vec(), bmp.width as _, bmp.rows as _, bmp.pitch as _))
        };

        Ok(Glyph {
            ft_glyph_slot,
            u8_monochrome_bitmap,
        })
    }
}


// FIXME: destructure dumb FT_GlyphSlotRec so we can derive stuff
// #[derive(Debug, Hash, PartialEq, Eq)]
pub struct Glyph {
    ft_glyph_slot: FT_GlyphSlotRec,
    u8_monochrome_bitmap: Option<ImgVec<u8>>,
}

impl Glyph {
    fn g(&self) -> &FT_GlyphSlotRec {
        &self.ft_glyph_slot
    }
    fn m(&self) -> &FT_Glyph_Metrics {
        &self.g().metrics
    }
    pub fn format(&self) -> GlyphFormat { GlyphFormat::from_ft_format(self.g().format).unwrap() }

    pub fn size(&self) -> Extent2<f64> { Extent2::new(self.m().width, self.m().height).map(f64_from_26_6) }
    pub fn horizontal_bearing(&self) -> Vec2<f64> { Vec2::new(self.m().horiBearingX, self.m().horiBearingY).map(f64_from_26_6) }
    pub fn horizontal_advance(&self) -> f64 { f64_from_26_6(self.m().horiAdvance) }
    pub fn vertical_bearing(&self) -> Vec2<f64> { Vec2::new(self.m().vertBearingX, self.m().vertBearingY).map(f64_from_26_6) }
    pub fn vertical_advance(&self) -> f64 { f64_from_26_6(self.m().vertAdvance) }

    pub fn linear_advance(&self) -> Vec2<f64> { Vec2::new(self.g().linearHoriAdvance, self.g().linearVertAdvance).map(f64_from_16_16) }
    pub fn advance(&self) -> Vec2<f64> { Vec2::new(self.g().advance.x, self.g().advance.y).map(f64_from_26_6) }

    pub fn u8_monochrome_bitmap(&self) -> Option<ImgRef<u8>> { self.u8_monochrome_bitmap.as_ref().map(ImgVec::as_ref) }
    /// The bitmap's left and top bearing expressed in integer pixels. The top bearing is the distance from the baseline to the top-most glyph scanline, upwards y coordinates being positive.
    pub fn bitmap_bearing(&self) -> Vec2<i32> { Vec2::new(self.g().bitmap_left, self.g().bitmap_top) }

    pub fn outline(&self) -> Option<Outline> { unimplemented!{} }
}

#[derive(Debug)]
pub struct Outline {
    ft_outline: FT_Outline,
}

impl Drop for Outline {
    fn drop(&mut self) {
        let &FT_Outline {
            n_contours, n_points,
            points, tags, contours,
            flags,
        } = &self.ft_outline;
        drop(Box::from_raw(points)) // FIXME: ptr to slice, not T
    }
}

impl Outline {
    pub(crate) fn new(ft_outline: &FT_Outline) -> Self {
        let &FT_Outline {
            n_contours, n_points,
            points, tags, contours,
            flags,
        } = ft_outline;
        let foo = Box::new(slice).into_raw()
        Self {
            ft_outline: FT_Outline {
                n_contours, n_points, flags,
                points, tags, contours,
            }
        }
    }
    pub fn translate(&mut self, t: Vec<f64>) {
        let t = t.map(f64_to_26_6);
        unsafe {
            FT_Outline_Translate(self.ft_outline, t.x, t.y);
        }
    }
    pub fn transform_2x2(&mut self, m: Mat2<f64>) {
        let m = m.map(f64_to_26_6);
        let m = FT_Matrix {
            xx: m[(0, 0)],
            xy: m[(0, 1)],
            yx: m[(1, 0)],
            yy: m[(1, 1)],
        };
        unsafe {
            FT_Outline_Transform(self.ft_outline, &m);
        }
    }
    pub fn embolden(&mut self, strength: Vec<f64>) {
        let strength = strength.map(f64_to_26_6);
        unsafe {
            FT_Outline_EmboldenXY(self.ft_outline, strength.x, strength.y);
        }
    }
    pub fn ctrl_box(&self) -> Aabr<f64> {
        unsafe {
            let mut ft_bbox = mem::uninitialized();
            FT_Outline_get_CBox(self.ft_outline, &mut ft_bbox);
            aabr_from_ft_bbox(ft_bbox)
        }
    }
    pub fn bounding_box(&self) -> Aabr<f64> {
        unsafe {
            let mut ft_bbox = mem::uninitialized();
            FT_Outline_get_BBox(self.ft_outline, &mut ft_bbox);
            aabr_from_ft_bbox(ft_bbox)
        }
    }
    pub fn fill_rule(&self) -> Option<OutlineFillRules> {
        match unsafe { FT_Outline_Get_Orientation(self.ft_outline) } {
            freetype_sys::FT_ORIENTATION_FILL_RIGHT => Some(OutlineFill::ClockwiseContours),
            freetype_sys::FT_ORIENTATION_FILL_RIGHT => Some(OutlineFill::CounterClockwiseContours),
            freetype_sys::FT_ORIENTATION_NONE => None,
            _ => unreachable!(),
        }
    }
    pub fn decompose(&self, decomposer: &mut OutlineDecomposer) {
        unsafe fn move_to(to: *const FT_Vector, user: *mut c_void) {
            let decomposer = &mut *(user as *mut OutlineDecomposer);
            decomposer.move_to(Vec2::new((*to).x, (*to).y).map(f64_from_26_6));
        }
        unsafe fn line_to(to: *const FT_Vector, user: *mut c_void) {
            let decomposer = &mut *(user as *mut OutlineDecomposer);
            decomposer.line_to(Vec2::new((*to).x, (*to).y).map(f64_from_26_6));
        }
        unsafe fn conic_to(ctrl: *const FT_Vector, end: *const FT_Vector, user: *mut c_void) {
            let decomposer = &mut *(user as *mut OutlineDecomposer);
            let ctrl = Vec2::new((*ctrl).x, (*ctrl).y).map(f64_from_26_6);
            let end = Vec2::new((*end).x, (*end).y).map(f64_from_26_6);
            decomposer.conic_to(ctrl, end);
        }
        unsafe fn cubic_to(ctrl0: *const FT_Vector, ctrl1: *const FT_Vector, end: *const FT_Vector, user: *mut c_void) {
            let decomposer = &mut *(user as *mut OutlineDecomposer);
            let ctrl0 = Vec2::new((*ctrl0).x, (*ctrl0).y).map(f64_from_26_6);
            let ctrl1 = Vec2::new((*ctrl1).x, (*ctrl1).y).map(f64_from_26_6);
            let end = Vec2::new((*end).x, (*end).y).map(f64_from_26_6);
            decomposer.cubic_to(ctrl0, ctrl1, end);
        }
        let ft_outline_funcs = FT_Outline_Funcs { move_to, line_to, conic_to, cubic_to };
        unsafe {
            FT_Outline_Decompose(self.ft_outline, &ft_outline_funcs, decomposer);
        }
    }
}

pub trait OutlineDecomposer {
    fn move_to(&mut self, to: Vec2<f64>);
    fn line_to(&mut self, to: Vec2<f64>);
    fn conic_to(&mut self, ctrl: Vec2<f64>, end: Vec2<f64>);
    fn cubic_to(&mut self, ctrl0: Vec2<f64>, ctrl1: Vec2<f64>, end: Vec2<f64>);
}


#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum OutlineFillRules {
    ClockwiseContours = FT_ORIENTATION_FILL_RIGHT,
    CounterClockwiseContours = FT_ORIENTATION_FILL_LEFT,
}

fn aabr_from_ft_bbox(ft_bbox: FT_BBox) -> Aabr<f64> {
    let FT_BBox { xMin, xMax, yMin, yMax } = ft_bbox;
    Aabr {
        min: Vec2::new(xMin, xMax),
        max: Vec2::new(yMin, yMax),
    }.map(f64_from_26_6)
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GlyphFormat {
    /// The glyph image is a composite of several other images. This format is only used with FT_LOAD_NO_RECURSE, and is used to report compound glyphs (like accented characters).
    Composite = FT_GLYPH_FORMAT_COMPOSITE,

    /// The glyph image is a bitmap, and can be described as an FT_Bitmap. You generally need to access the ‘bitmap’ field of the FT_GlyphSlotRec structure to read it.
    Bitmap = FT_GLYPH_FORMAT_BITMAP,

    /// The glyph image is a vectorial outline made of line segments and Bezier arcs; it can be described as an FT_Outline; you generally want to access the ‘outline’ field of the FT_GlyphSlotRec structure to read it.
    Outline = FT_GLYPH_FORMAT_OUTLINE,

    /// The glyph image is a vectorial path with no inside and outside contours. Some Type 1 fonts, like those in the Hershey family, contain glyphs in this format. These are described as FT_Outline, but FreeType isn't currently capable of rendering them correctly.
    Plotter = FT_GLYPH_FORMAT_PLOTTER,
}

impl GlyphFormat {
    fn from_ft_format(ft_format: u32) -> Option<Self> {
        match ft_format {
            freetype_sys::FT_GLYPH_FORMAT_COMPOSITE => Some(GlyphFormat::Composite),
            freetype_sys::FT_GLYPH_FORMAT_BITMAP    => Some(GlyphFormat::Bitmap),
            freetype_sys::FT_GLYPH_FORMAT_OUTLINE   => Some(GlyphFormat::Outline),
            freetype_sys::FT_GLYPH_FORMAT_PLOTTER   => Some(GlyphFormat::Plotter),
            _ => None
        }
    }
}
