use mesh::vertex_array::VertexArray;
use mesh::index_array::IndexArray;
use mesh::text::{Program, Vertex};
use gx;
use font::Font;
use v::Vec2;

// pub struct TextInstance { position, color... }
// Anything pixel-perfect has to be rendered via a camera.

// This is a convenience for both on-CPU and on-GPU representations for drawn text.
// There may be multiple _instances_.
#[derive(Debug)]
pub struct Text {
    pub string: String,
    pub vertices: VertexArray<Program>,
    pub indices: IndexArray<u16>,
}

impl Text {
    pub fn new(prog: &Program, label: &str) -> Self {
        Self {
            string: String::default(),
            vertices: VertexArray::from_vertices(prog, label, gx::BufferUsage::DynamicDraw, vec![]),
            indices: IndexArray::from_indices(label, gx::BufferUsage::DynamicDraw, vec![]),
        }
    }
    pub fn update_gl(&mut self, font: &Font) {
        let atlas_size = font.texture_size.map(|x| x as f32);
        let mut cur = Vec2::<i16>::zero();
        let mut i = 0;

        self.vertices.vertices.clear();
        self.indices.indices.clear();

        for c in self.string.chars() {
            match c {
                '\n' => {
                    cur.x = 0;
                    cur.y += font.height as i16;
                    continue;
                },
                ' ' => {
                    cur += font.glyph_info[&' '].advance;
                    continue;
                },
                '\t' => {
                    cur += font.glyph_info[&' '].advance * 4;
                    continue;
                },
                c if c.is_ascii_control() || c.is_ascii_whitespace() => {
                    continue;
                },
                _ => (),
            };
            let c = if font.glyph_info.contains_key(&c) { c } else { '?' };
            let glyph = &font.glyph_info[&c];
            let mut texcoords = glyph.bounds.into_rect().map(
                |p| p as f32,
                |e| e as f32
            );
            texcoords.x /= atlas_size.w;
            texcoords.y /= atlas_size.h;
            texcoords.w /= atlas_size.w;
            texcoords.h /= atlas_size.h;

            let offset = glyph.offset.map(|x| x as f32) / atlas_size;
            let mut world_cur = cur.map(|x| x as f32) / atlas_size;
            world_cur.y = -world_cur.y;
            world_cur.x += offset.x;
            world_cur.y -= texcoords.h - offset.y;

            let bottom_left = Vertex {
                position: world_cur,
                texcoords: texcoords.position() + Vec2::unit_y() * texcoords.h,
            };
            let bottom_right = Vertex {
                position: world_cur + Vec2::unit_x() * texcoords.w,
                texcoords: texcoords.position() + texcoords.extent(),
            };
            let top_left = Vertex {
                position: world_cur + Vec2::unit_y() * texcoords.h,
                texcoords: texcoords.position(),
            };
            let top_right = Vertex {
                position: world_cur + texcoords.extent(),
                texcoords: texcoords.position() + Vec2::unit_x() * texcoords.w,
            };
            self.vertices.vertices.push(bottom_left);
            self.vertices.vertices.push(bottom_right);
            self.vertices.vertices.push(top_left);
            self.vertices.vertices.push(top_right);
            self.indices.indices.push(i*4 + 0);
            self.indices.indices.push(i*4 + 1);
            self.indices.indices.push(i*4 + 2);
            self.indices.indices.push(i*4 + 3);
            self.indices.indices.push(i*4 + 2);
            self.indices.indices.push(i*4 + 1);

            cur += glyph.advance;
            i += 1;
        }

        self.vertices.update_and_resize_vbo();
        self.indices.update_and_resize_ibo();
    }
}
