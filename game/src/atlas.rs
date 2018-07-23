use std::collections::HashMap;
use fate::font::Font;
use fate::img::ImgVec;
use fate::math::{Vec2, Aabr};

// Greyscale mono
pub struct Atlas {
    pub atlas: ImgVec<u8>,
    pub glyphs: HashMap<char, GlyphInfo>,
}

pub struct GlyphInfo {
    pub bounds: Aabr<u16>,
    pub offset: Vec2<i16>,
    pub advance: Vec2<i16>,
}

// TODO: Integrate in fate-font
impl Atlas {
    pub fn load(font: &mut Font, chars: &str, tex_side: usize) -> Self {
        assert!(tex_side.is_power_of_two());
        let mut atlas = ImgVec::new(vec!(0_u8; tex_side * tex_side), tex_side, tex_side);
        let mut glyphs = HashMap::new();

        let mut pen = Vec2::<usize>::zero();

        for c in chars.chars() {
            let glyph = font.glyph(c).pedantic().render_u8_monochrome_bitmap().load().unwrap();
            let bmp = glyph.u8_monochrome_bitmap().unwrap();
            if pen.y + glyph.size_px().h as usize + 1 >= tex_side {
                panic!();
            }
            if pen.x + bmp.width() >= tex_side {
                pen.x = 0;
                pen.y += 1 + glyph.size_px().h as usize;
            }
            for row in 0 .. bmp.height() {
                for col in 0 .. bmp.width() {
                    let x = pen.x + col;
                    let y = pen.y + row;
                    atlas[(x, y)] = bmp[(col, row)];
                }
            }

            let gi = GlyphInfo {
                bounds: Aabr {
                    min: pen.map(|x| x as _),
                    max: (pen + Vec2::new(bmp.width() as _, bmp.height() as _)).map(|x| x as _),
                },
                offset: glyph.bitmap_bearing().map(|x| x as _),
                advance: glyph.advance_px().map(|x| x as _),
            };
            let old = glyphs.insert(c, gi);
            assert!(old.is_none());

            pen.x += bmp.width() + 1;
        }

        Self {
            atlas,
            glyphs,
        }
    }
    pub fn all_supported_chars() -> String {
        // Do include space. We only care about its GlyphInfo, so it shouldn't
        // have its place in the atlas, but: deadlines!!
        let mut chars = " ".to_string();
        // All printable ASCII chars...
        for i in 33_u8..127_u8 {
            chars.push(i as char);
        }
        // Hon hon hon Baguette Au Jambon
        chars += "°éèçàù";
        chars
    }
}