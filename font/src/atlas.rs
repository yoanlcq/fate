use std::collections::HashMap;
use img::ImgVec;
use math::{Vec2, Aabr};
use super::Font;

// Greyscale mono
#[derive(Debug, Clone, PartialEq)]
pub struct Atlas {
    pub img: ImgVec<u8>,
    pub glyphs: HashMap<char, AtlasGlyphInfo>,
    pen: Vec2<usize>,
    biggest_height_in_line: usize,
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct AtlasGlyphInfo {
    pub bounds: Aabr<u16>,
    pub offset: Vec2<i16>,
    pub advance: Vec2<i16>,
}

impl Atlas {
    pub fn new(tex_side: usize) -> Self {
        assert!(tex_side.is_power_of_two());
        Self {
            img: ImgVec::new(vec!(0_u8; tex_side * tex_side), tex_side, tex_side),
            glyphs: HashMap::new(),
            pen: Vec2::zero(),
            biggest_height_in_line: 0,
        }
    }
    pub fn with_font_chars<I: IntoIterator<Item=char>>(tex_side: usize, font: &Font, chars: I) -> Self {
        let mut me = Self::new(tex_side);
        me.add_chars(font, chars);
        me
    }
    pub fn add_chars<I: IntoIterator<Item=char>>(&mut self, font: &Font, chars: I) {
        for c in chars {
            self.add_char(font, c);
        }
    }
    pub fn add_char(&mut self, font: &Font, c: char) {
        let glyph = font.glyph(c).pedantic().render_u8_monochrome_bitmap().load().unwrap();
        let bmp = glyph.u8_monochrome_bitmap();
        let (bmp_w, bmp_h) = bmp.map(|x| (x.width(), x.height())).unwrap_or((0, 0));

        if self.pen.y + glyph.size_px().h as usize + 1 >= self.img.height() {
            panic!();
        }

        if self.pen.x + bmp_w >= self.img.width() {
            self.pen.x = 0;
            self.pen.y += 1 + self.biggest_height_in_line;
            self.biggest_height_in_line = 0;
        }

        self.biggest_height_in_line = ::std::cmp::max(bmp_h, self.biggest_height_in_line);

        if let Some(bmp) = bmp {
            for row in 0 .. bmp_h {
                for col in 0 .. bmp_w {
                    let x = self.pen.x + col;
                    let y = self.pen.y + row;
                    self.img[(x, y)] = bmp[(col, row)]; // FIXME: Be less braindead and use memcpy()
                }
            }
        }

        let gi = AtlasGlyphInfo {
            bounds: Aabr {
                min: self.pen.map(|x| x as _),
                max: (self.pen + Vec2::new(bmp_w as _, bmp_h as _)).map(|x| x as _),
            },
            offset: glyph.bitmap_bearing().map(|x| x as _),
            advance: glyph.advance_px().map(|x| x as _),
        };
        let old = self.glyphs.insert(c, gi);
        assert!(old.is_none());

        self.pen.x += bmp_w + 1;
    }
}
