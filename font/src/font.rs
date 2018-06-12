use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::error::Error;
use std::ffi::CString;
use std::ptr;
use std::mem;
use std::slice;
use std::mem::ManuallyDrop;
use vek::{Vec2, Extent2, Aabr};
use freetype_sys::*;

macro_rules! ft_error_codes {
    ($($variant:ident)+) => {
        #[repr(i32)]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum FreeTypeError {
            $($variant = freetype_sys::$variant),+
        }
        impl FreeTypeError {
            pub fn try_from_i32(i: i32) -> Option<Self> {
                match i {
                    freetype_sys::$variant => Some(FreeTypeError::$variant),
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
    FT_Err_Ok	
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

fn ft_result(status: i32) -> Result<(), FreeTypeError> {
    if status == FT_Err_Ok {
        Ok()
    } else {
        Err(FreeTypeError::try_from_i32(status).unwrap())
    }
}


#[derive(Debug)]
struct FreeType {
    ft_library: FT_Library,
}

impl !Send for Freetype {}

impl Drop for FreeType {
    fn drop(&mut self) {
        unsafe {
            FT_Done_FreeType(self.0);
        }
    }
}

#[derive(Debug)]
pub struct FontLoader {
    ft: Arc<Freetype>,
}

#[derive(Debug)]
pub struct Font {
    ft: Arc<FreeType>,
    ft_face: FT_Face,
}

impl Drop for Font {
    fn drop(&mut self) {
        unsafe {
            FT_Done_Face(self.ft_face);
        }
    }
}


// NOTE: We store a lot of them, so I prefer to use 16-bit integers here.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct GlyphInfo {
    // NOTE: Y axis goes downwards!
    pub bounds: Aabr<u16>,
    // Horizontal position relative to the cursor, in pixels.
    // Vertical position relative to the baseline, in pixels.
    pub offset: Vec2<i16>,
    // How far to move the cursor for the next character.
    pub advance: Vec2<i16>,
}


impl FontLoader {
    pub fn new() -> Result<Self, FreeTypeError> {
        let mut ft_library: FT_Library = unsafe { mem::uninitialized() };
        ft_result(unsafe { FT_Init_FreeType(&mut ft_library) })?;
        Ok(Self { ft: Arc::new(FreeType { ft_library })})
    }

    pub fn create_font(&self, mem: &[u8], desired_height: u32) -> Result<Font, String> {
        let mut ft_face: FT_Face = unsafe { mem::uninitialized() };
        ft_result(FT_New_Memory_Face(self.ft.ft_library, mem.as_ptr(), mem.len(), 0, &mut ft_face))?;
        unsafe {
            let (width, height) = (0, font_size);
            ft_result(FT_Set_Pixel_Sizes(ft_face, width, desired_height))?;
        }
        let font = Font {
            ft: Arc::clone(&self.ft),
            ft_face,
        };
        Ok(font)
    }
}

impl Font {
    fn height(&self) -> u32 {
        let metrics = unsafe { &(*(*self.ft_face).size).metrics };
        metrics.height / 64
    }
    pub fn glyph_pedantic(&self, c: char) -> Result<Glyph, String> {
        self.glyph_with_flags(c, FT_LOAD_PEDANTIC)
    }
    pub fn glyph(&self, c: char) -> Result<Glyph, String> {
        self.glyph_with_flags(c, FT_LOAD_DEFAULT)
    }
    fn glyph_with_flags(&self, c: char, flags: u32) -> Result<Glyph, String> {
        if unsafe { FT_Load_Char(self.ft_face, c as u64, flags) } != 0 {
            return Err(format!("Could not load character '{}'", c)); // FIXME: FreeType error code!
        }
        let glyph_slot = unsafe { &*(*self.ft_face).glyph };
        let &FT_GlyphSlotRec_ {
            library: _, face: _, next: _, reserved: _, generic: _,
            metrics, linearHoriAdvance, linearVertAdvance, advance,
            format,
            bitmap, bitmap_left, bitmap_top,
            outline,
            num_subglyphs: _, subglyphs: _,
            control_data: _, control_len: _,
            lsb_delta: _, rsb_delta: _,
            other: _,
            internal: _,
        } = glyph_slot;

        // FT_RENDER_MODE_MONO   => 1-bit bitmaps
        // FT_RENDER_MODE_NORMAL => 8-bit anti-aliased
        // FT_RENDER_MODE_LCD    => Horizontal RGB and BGR
        // FT_RENDER_MODE_LCD_V  => Vertical RGB and BGR
        FT_Render_Glyph(glyph_slot, FT_RENDER_MODE_NORMAL);

        match format {
            FT_GLYPH_FORMAT_BITMAP => {
                let bmp = &g.bitmap;
                let bmp = unsafe {
                    slice::from_raw_parts(bmp.buffer, (bmp.rows*bmp.pitch) as usize)
                };
                let bmp = bmp.to_vec();
                bitmap_left;
                bitmap_top;
            },
            FT_GLYPH_FORMAT_OUTLINE => {
                outline;
            },
            FT_GLYPH_FORMAT_COMPOSITE => {
                num_subglyphs;
                subglyphs;
            },
            FT_GLYPH_FORMAT_PLOTTER => (),
            FT_GLYPH_FORMAT_NONE => (),
            _ => (),
        };
    }
}

#[derive(Debug)]
pub struct FontAtlas {
    img: Vec<u8>,
}

impl FontAtlas {
    pub fn new(size: Extent2<u32>) -> Self {
        Self { img: vec![0; (size.w * size.h) as usize] }
    }
    pub fn add_char(&mut self, g: &Glyph) {
        let metrics = unsafe { &(*(*face).size).metrics };
        // Partly taken from https://gist.github.com/baines/b0f9e4be04ba4e6f56cab82eef5008ff

        let size = self.size;

        for c in chars.chars() {
            if pen.y + (metrics.height / 64) as usize + 1 >= size.h {
                panic!("Couldn't create font atlas for `{}`: {}x{} is not large enough!", path.display(), size.w, size.h);
            }
            if pen.x + bmp.width as usize >= size.w {
                pen.x = 0;
                pen.y += (metrics.height / 64) as usize + 1;
            }

            for row in 0..(bmp.rows as usize) {
                for col in 0..(bmp.width as usize) {
                    let x = pen.x + col;
                    let y = pen.y + row;
                    pixels[y * size.w + x] = bmp_buffer[row * (bmp.pitch as usize) + col];
                }
            }

            let gi = AtlasGlyphInfo {
                bounds: Aabr {
                    min: pen.map(|x| x as _),
                    max: (pen + Vec2::new(bmp.width as _, bmp.rows as _)).map(|x| x as _),
                },
                offset: Vec2::new(g.bitmap_left as _, g.bitmap_top as _),
                advance: Vec2::new(g.advance.x, g.advance.y).map(|x| (x / 64) as _),
            };

            pen.x += bmp.width as usize + 1;
        }
    }
}

