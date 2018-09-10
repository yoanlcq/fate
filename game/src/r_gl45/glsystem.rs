use fate::math::{Extent2, Rgba, Rect};
use fate::gx::{self, gl::{self, types::*}};

use gpu::GpuCmd;
use viewport::{ViewportVisitor, AcceptLeafViewport};
use cubemap::{CubemapArrayID};
use texture2d::Texture2DArrayID;
use mesh::VertexAttribIndex;
use system::*;

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(C)]
pub struct GLDrawElementsIndirectCommand {
    pub nb_indices: GLuint,
    pub nb_instances: GLuint,
    pub first_index: GLuint,
    pub base_vertex: GLuint,
    pub base_instance: GLuint,
}

unsafe fn toast_rendering() {
    // Create the VAO;
    let vao = 0;
    let position_vbo = 0;
    let normal_vbo = 0;
    let uv_vbo = 0;
    let model_matrix_vbo = 0;
    let material_index_vbo = 0;
    let ibo = 0;

    gl::BindVertexArray(vao);
    gl::EnableVertexAttribArray(VertexAttribIndex::Position as _);
    gl::EnableVertexAttribArray(VertexAttribIndex::Normal as _);
    gl::EnableVertexAttribArray(VertexAttribIndex::UV as _);
    gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 0);
    gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 1);
    gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 2);
    gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 3);
    gl::EnableVertexAttribArray(VertexAttribIndex::MaterialIndex as _);

    gl::VertexAttribDivisor(VertexAttribIndex::Position as _, 0);
    gl::VertexAttribDivisor(VertexAttribIndex::Normal as _, 0);
    gl::VertexAttribDivisor(VertexAttribIndex::UV as _, 0);
    gl::VertexAttribDivisor(VertexAttribIndex::ModelMatrix as GLuint + 0, 1);
    gl::VertexAttribDivisor(VertexAttribIndex::ModelMatrix as GLuint + 1, 1);
    gl::VertexAttribDivisor(VertexAttribIndex::ModelMatrix as GLuint + 2, 1);
    gl::VertexAttribDivisor(VertexAttribIndex::ModelMatrix as GLuint + 3, 1);
    gl::VertexAttribDivisor(VertexAttribIndex::MaterialIndex as _, 1);

    gl::BindBuffer(gl::ARRAY_BUFFER, position_vbo);
    gl::VertexAttribPointer(VertexAttribIndex::Position as _, 3, gl::FLOAT, gl::FALSE, 0, 0 as _);
    gl::BindBuffer(gl::ARRAY_BUFFER, normal_vbo);
    gl::VertexAttribPointer(VertexAttribIndex::Normal as _, 3, gl::FLOAT, gl::FALSE, 0, 0 as _);
    gl::BindBuffer(gl::ARRAY_BUFFER, uv_vbo);
    gl::VertexAttribPointer(VertexAttribIndex::UV as _, 2, gl::FLOAT, gl::FALSE, 0, 0 as _);
    gl::BindBuffer(gl::ARRAY_BUFFER, model_matrix_vbo);
    gl::VertexAttribPointer(VertexAttribIndex::ModelMatrix as GLuint + 0, 4, gl::FLOAT, gl::FALSE, 4*4*4, (0*4*4) as _);
    gl::VertexAttribPointer(VertexAttribIndex::ModelMatrix as GLuint + 1, 4, gl::FLOAT, gl::FALSE, 4*4*4, (1*4*4) as _);
    gl::VertexAttribPointer(VertexAttribIndex::ModelMatrix as GLuint + 2, 4, gl::FLOAT, gl::FALSE, 4*4*4, (2*4*4) as _);
    gl::VertexAttribPointer(VertexAttribIndex::ModelMatrix as GLuint + 3, 4, gl::FLOAT, gl::FALSE, 4*4*4, (3*4*4) as _);
    gl::BindBuffer(gl::ARRAY_BUFFER, material_index_vbo);
    gl::VertexAttribIPointer(VertexAttribIndex::MaterialIndex as _, 1, gl::UNSIGNED_SHORT, 0, 0 as _);
    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    gl::BindVertexArray(0);

    // Drawing

    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
    gl::BindVertexArray(vao);

    let mut cmds = vec![];
    let mut base_vertex = 0; // Added to each element of `indices`.
    for _ in 0..1 {
        let nb_indices = 0; // TODO
        let nb_instances = 0; // TODO
        cmds.push(GLDrawElementsIndirectCommand {
            nb_indices,
            nb_instances,
            first_index: base_vertex, // Offset into the index buffer
            base_vertex,
            base_instance: 0,
        });
        base_vertex += nb_indices;
    }
    gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, 0); // read from cpu memory

    gl::MultiDrawElementsIndirect(gx::Topology::Triangles as _, gl::UNSIGNED_INT, cmds.as_ptr() as _, cmds.len() as _, 0);

    gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, 0);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
    gl::BindVertexArray(0);
}


#[derive(Debug)]
pub struct GLSystem {
    cubemap_arrays: [GLuint; CubemapArrayID::MAX],
    texture2d_arrays: [GLuint; Texture2DArrayID::MAX],
}

impl GLSystem {
    pub fn new() -> Self {
        let mut cubemap_arrays = [0; CubemapArrayID::MAX];
        let mut texture2d_arrays = [0; CubemapArrayID::MAX];
        unsafe {
            gl::GenTextures(cubemap_arrays.len() as _, cubemap_arrays.as_mut_ptr());
            gl::GenTextures(texture2d_arrays.len() as _, texture2d_arrays.as_mut_ptr());

            for tex in cubemap_arrays.iter() {
                assert_ne!(*tex, 0);
                gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, *tex);
            }
            gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, 0);

            for tex in texture2d_arrays.iter() {
                assert_ne!(*tex, 0);
                gl::BindTexture(gl::TEXTURE_2D_ARRAY, *tex);
            }
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
        }
        Self {
            cubemap_arrays, texture2d_arrays,
        }
    }
}

impl Drop for GLSystem {
    fn drop(&mut self) {
        let &mut Self {
            ref mut cubemap_arrays,
            ref mut texture2d_arrays,
        } = self;
        unsafe {
            gl::DeleteTextures(cubemap_arrays.len() as _, cubemap_arrays.as_mut_ptr());
            gl::DeleteTextures(texture2d_arrays.len() as _, texture2d_arrays.as_mut_ptr());
        }
    }
}

impl System for GLSystem {
    fn draw(&mut self, g: &mut G, _d: &Draw) {
        self.process_gpu_cmd_queue(g);

        let Extent2 { w, h } = g.input.canvas_size();
        unsafe {
            gl::Viewport(0, 0, w as _, h as _);
            let Rgba { r, g, b, a } = g.viewport_db().border_color();
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        g.visit_viewports(self);
    }
}

impl GLSystem {
    fn process_gpu_cmd_queue(&mut self, g: &G) {
        for cmd in g.gpu_cmd_queue() {
            self.process_gpu_cmd(g, cmd);
        }
    }
    fn process_gpu_cmd(&mut self, g: &G, cmd: &GpuCmd) {
        unsafe {
            match *cmd {
                GpuCmd::ClearColorEdit => {
                    let Rgba { r, g, b, a } = g.clear_color();
                    gl::ClearColor(r, g, b, a);
                },
                GpuCmd::Texture2DArrayCreate(id) => {
                    let info = g.texture2d_array_info(id).unwrap();
                    gl::TextureStorage3D(self.texture2d_arrays[id.0 as usize], info.levels as _, info.internal_format as _, info.size.w as _, info.size.h as _, info.nb_textures as _);
                },
                GpuCmd::CubemapArrayCreate(id) => {
                    let info = g.cubemap_array_info(id).unwrap();
                    gl::TextureStorage3D(self.cubemap_arrays[id.0 as usize], info.levels as _, info.internal_format as _, info.size.w as _, info.size.h as _, (info.nb_cubemaps * 6) as _);
                },

                GpuCmd::Texture2DArrayDelete(id) => {
                    let tex = &mut self.texture2d_arrays[id.0 as usize];
                    gl::DeleteTextures(1, tex);
                    gl::GenTextures(1, tex);
                    assert_ne!(*tex, 0);
                    gl::BindTexture(gl::TEXTURE_2D_ARRAY, *tex);
                    gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
                },
                GpuCmd::CubemapArrayDelete(id) => {
                    let tex = &mut self.cubemap_arrays[id.0 as usize];
                    gl::DeleteTextures(1, tex);
                    gl::GenTextures(1, tex);
                    assert_ne!(*tex, 0);
                    gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, *tex);
                    gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, 0);
                },


                GpuCmd::Texture2DArrayClear(id, level, color) => {
                    let color: Rgba<f32> = color; // Assert that we're dealing with the correct type
                    gl::ClearTexImage(self.texture2d_arrays[id.0 as usize], level as _, gl::RGBA, gl::FLOAT, color.as_ptr() as _);
                },
                GpuCmd::CubemapArrayClear(id, level, color) => {
                    let color: Rgba<f32> = color; // Assert that we're dealing with the correct type
                    gl::ClearTexImage(self.cubemap_arrays[id.0 as usize], level as _, gl::RGBA, gl::FLOAT, color.as_ptr() as _);
                },

                GpuCmd::Texture2DArraySubImage2D(id, slot, ref img) => {
                    let z = slot;
                    let depth = 1;
                    gl::TextureSubImage3D(self.cubemap_arrays[id.0 as usize], img.level as _, img.x as _, img.y as _, z as _, img.w as _, img.h as _, depth, img.format as _, img.type_ as _, img.data.as_ptr() as _);
                },
                GpuCmd::CubemapArraySubImage2D(id, slot, face, ref img) => {
                    let z = slot * 6 + face as usize;
                    let depth = 1;
                    gl::TextureSubImage3D(self.cubemap_arrays[id.0 as usize], img.level as _, img.x as _, img.y as _, z as _, img.w as _, img.h as _, depth, img.format as _, img.type_ as _, img.data.as_ptr() as _);
                },
            }
        }
    }
}

impl ViewportVisitor for GLSystem {
    fn accept_leaf_viewport(&mut self, args: AcceptLeafViewport) {
        unsafe {
            let Rect { x, y, w, h } = args.rect;
            gl::Viewport(x as _, y as _, w as _, h as _);

            // Temporary
            gl::Enable(gl::SCISSOR_TEST);

            let (bx, by) = (args.border_px, args.border_px);
            if w < bx+bx || h < by+by {
                return;
            }
            let (x, y, w, h) = (x+bx, y+by, w-bx-bx, h-by-by);
            let Rgba { r, g, b, a } = args.info.clear_color;
            gl::Scissor(x as _, y as _, w as _, h as _);
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT/* | gl::DEPTH_BUFFER_BIT*/);

            gl::Disable(gl::SCISSOR_TEST);
        }
    }
}
