use fate::math::{Vec3, Rgba};
use fate::gx::gl::{self, types::GLenum}; // FIXME: Unrelated to GL

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub topology: GLenum,
    pub vposition: Vec<Vec3<f32>>, // Not optional
    pub vnormal: Vec<Vec3<f32>>, // Not optional
    pub vcolor: Vec<Rgba<u8>>, // Optional. If there's only one element, it is used for all vertices.
    pub indices: Vec<u16>, // Optional. If empty, it's rendered using glDrawArrays.
}

impl Mesh {
    pub fn new_icosahedron(s: f32, nb_subdivisions: usize) -> Self {
        let t = (1. + 5_f32.sqrt()) / 2.;
        let mut vertices = vec![
            Vec3::new(-1.0,  t, 0.0).normalized() * s,
            Vec3::new( 1.0,  t, 0.0).normalized() * s,
            Vec3::new(-1.0, -t, 0.0).normalized() * s,
            Vec3::new( 1.0, -t, 0.0).normalized() * s,
            Vec3::new(0.0, -1.0,  t).normalized() * s,
            Vec3::new(0.0,  1.0,  t).normalized() * s,
            Vec3::new(0.0, -1.0, -t).normalized() * s,
            Vec3::new(0.0,  1.0, -t).normalized() * s,
            Vec3::new( t, 0.0, -1.0).normalized() * s,
            Vec3::new( t, 0.0,  1.0).normalized() * s,
            Vec3::new(-t, 0.0, -1.0).normalized() * s,
            Vec3::new(-t, 0.0,  1.0).normalized() * s,
        ];
        let mut indices = vec![
            0, 11, 5,
            0, 5, 1,
            0, 1, 7,
            0, 7, 10,
            0, 10, 11,
            1, 5, 9,
            5, 11, 4,
            11, 10, 2,
            10, 7, 6,
            7, 1, 8,
            3, 9, 4,
            3, 4, 2,
            3, 2, 6,
            3, 6, 8,
            3, 8, 9,
            4, 9, 5,
            2, 4, 11,
            6, 2, 10,
            8, 6, 7,
            9, 8, 1,
        ];

        for _ in 0..nb_subdivisions {
            let mut out_vertices = vec![];
            let mut out_indices = vec![];
            for face in indices.chunks(3) {
                let v0 = vertices[face[0] as usize];
                let v1 = vertices[face[1] as usize];
                let v2 = vertices[face[2] as usize];
                let v3 = ((v0 + v1) / 2.).normalized() * s;
                let v4 = ((v1 + v2) / 2.).normalized() * s;
                let v5 = ((v2 + v0) / 2.).normalized() * s;
                let i = out_vertices.len() as u16;
                out_vertices.extend(&[v0, v1, v2, v3, v4, v5]);
                out_indices.extend(&[i+0, i+3, i+5]);
                out_indices.extend(&[i+3, i+1, i+4]);
                out_indices.extend(&[i+5, i+4, i+2]);
                out_indices.extend(&[i+3, i+4, i+5]);
            }
            vertices = out_vertices;
            indices = out_indices;
        }

        Self {
            topology: gl::TRIANGLES,
            vposition: vertices.clone(),
            vnormal: vertices,
            vcolor: vec![Rgba::blue()],
            indices,
        }
    }

    // A skybox is special because face winding is inverted so that we don't need to change cull face state.
    pub fn new_skybox() -> Self {
        let mut m = Self::new_cube_smooth_triangle_strip(0.5);

        // Flip winding by inserting a degenerate triangle
        let pos = m.vposition[0];
        let norm = m.vnormal[0];
        m.vposition.insert(0, pos);
        m.vnormal.insert(0, norm);

        // ... and reverse normals too (not that they are expected to be used anyway...)
        for n in &mut m.vnormal {
            *n = -*n;
        }

        // Make sure to make it opaque white
        for col in &mut m.vcolor {
            *col = Rgba::white();
        }

        m
    }
    pub fn new_cube_smooth_triangle_strip(s: f32) -> Self {
        let vposition = vec![
            Vec3::new(-s,  s,  s), // Front-top-left
            Vec3::new( s,  s,  s), // Front-top-right
            Vec3::new(-s, -s,  s), // Front-bottom-left
            Vec3::new( s, -s,  s), // Front-bottom-right
            Vec3::new( s, -s, -s), // Back-bottom-right
            Vec3::new( s,  s,  s), // Front-top-right
            Vec3::new( s,  s, -s), // Back-top-right
            Vec3::new(-s,  s,  s), // Front-top-left
            Vec3::new(-s,  s, -s), // Back-top-left
            Vec3::new(-s, -s,  s), // Front-bottom-left
            Vec3::new(-s, -s, -s), // Back-bottom-left
            Vec3::new( s, -s, -s), // Back-bottom-right
            Vec3::new(-s,  s, -s), // Back-top-left
            Vec3::new( s,  s, -s), // Back-top-right
        ];

        Self {
            topology: gl::TRIANGLE_STRIP,
            vposition: vposition.clone(),
            vnormal: vposition,
            vcolor: vec![Rgba::red()],
            indices: vec![],
        }
    }
    pub fn new_cube_triangles(s: f32) -> Self {
        let v = (
            Vec3::new(-s,  s, -s), // 0
            Vec3::new( s,  s, -s), // 1
            Vec3::new( s,  s,  s), // 2
            Vec3::new(-s,  s,  s), // 3
            Vec3::new(-s, -s,  s), // 4
            Vec3::new(-s, -s, -s), // 5
            Vec3::new( s, -s, -s), // 6
            Vec3::new( s, -s,  s), // 7
        );
        let vposition = [
            v.7, v.2, v.1,
            v.7, v.1, v.6,
            v.4, v.5, v.0,
            v.4, v.0, v.3,
            v.0, v.1, v.2,
            v.0, v.2, v.3,
            v.5, v.4, v.7,
            v.5, v.7, v.6,
            v.4, v.3, v.2,
            v.4, v.2, v.7,
            v.1, v.0, v.5,
            v.1, v.5, v.6,
        ];
        let vnormal = [
            Vec3::right(),
            Vec3::right(),
            Vec3::right(),
            Vec3::right(),
            Vec3::right(),
            Vec3::right(),
            Vec3::left(),
            Vec3::left(),
            Vec3::left(),
            Vec3::left(),
            Vec3::left(),
            Vec3::left(),
            Vec3::up(),
            Vec3::up(),
            Vec3::up(),
            Vec3::up(),
            Vec3::up(),
            Vec3::up(),
            Vec3::down(),
            Vec3::down(),
            Vec3::down(),
            Vec3::down(),
            Vec3::down(),
            Vec3::down(),
            Vec3::forward_lh(),
            Vec3::forward_lh(),
            Vec3::forward_lh(),
            Vec3::forward_lh(),
            Vec3::forward_lh(),
            Vec3::forward_lh(),
            Vec3::back_lh(),
            Vec3::back_lh(),
            Vec3::back_lh(),
            Vec3::back_lh(),
            Vec3::back_lh(),
            Vec3::back_lh(),
        ];
        Self {
            topology: gl::TRIANGLES,
            vposition: vposition.to_vec(),
            vnormal: vnormal.to_vec(),
            vcolor: vec![Rgba::green()],
            indices: vec![],
        }
    }
    pub fn new_cube() -> Self {
        Self::new_cube_triangles(0.5)
    }
}