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
    use ::std::ptr;

    // Creating the resources
    
    let max_vertices = 0xffffff;
    let max_indices = 0xffffff;
    let max_instances = 0xffffff;

    let mut vao = 0;
    let mut buffers = [0; 6];

    gl::GenVertexArrays(1, &mut vao);
    gl::GenBuffers(buffers.len() as _, buffers.as_mut_ptr());

    let position_vbo = buffers[0];
    let normal_vbo = buffers[1];
    let uv_vbo = buffers[2];
    let model_matrix_vbo = buffers[3];
    let material_index_vbo = buffers[4];
    let ibo = buffers[5];

    let flags = gl::DYNAMIC_STORAGE_BIT;
    gl::NamedBufferStorage(position_vbo, max_vertices * 3 * 4, ptr::null(), flags);
    gl::NamedBufferStorage(normal_vbo, max_vertices * 3 * 4, ptr::null(), flags);
    gl::NamedBufferStorage(uv_vbo, max_vertices * 2 * 4, ptr::null(), flags);
    gl::NamedBufferStorage(model_matrix_vbo, max_instances * 4 * 4 * 4, ptr::null(), flags);
    gl::NamedBufferStorage(material_index_vbo, max_instances * 2, ptr::null(), flags);
    gl::NamedBufferStorage(ibo, max_indices * 4, ptr::null(), flags);

    // Specifying vertex attrib layout

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

    // TODO: Operations:
    // - Add mesh (grab chunks)
    // - Remove mesh (release chunks)
    // - Edit mesh (re-upload data)
    // - Add instance pack
    // - Remove instance pack
    // - Edit instance pack
    //
    // i.e
    // - Allocate N vertices
    // - Allocate N indices
    // - Allocate N instances
    // - Defragment the memory

    // Uploading data

    // gl::NamedBufferSubData(buf, offset, size, data);

    // Drawing

    let m = MemInfo::default();
    let mut cmds = vec![];

    for (i, mesh) in m.instance_ranges.iter().zip(m.instance_range_mesh_entry.iter()) {
        let index_range = &m.index_ranges[*mesh as usize];
        let vertex_range = &m.vertex_ranges[*mesh as usize];
        cmds.push(GLDrawElementsIndirectCommand {
            base_instance: i.start,
            nb_instances: i.end - i.start,
            first_index: index_range.start, // Offset into the index buffer
            nb_indices: index_range.end - index_range.start,
            base_vertex: vertex_range.start, // Value added to indices for vertex retrieval
        });
    }

    gl::BindVertexArray(vao);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
    gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, 0); // read from cpu memory
    gl::MultiDrawElementsIndirect(gx::Topology::Triangles as _, gl::UNSIGNED_INT, cmds.as_ptr() as _, cmds.len() as _, 0);
    gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, 0);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
    gl::BindVertexArray(0);
}

use ::std::ops::Range;
#[derive(Debug, Default, Clone, Hash, PartialEq)]
struct MemInfo {
    // Indexed by mesh
    pub vertex_ranges: Vec<Range<u32>>,
    pub index_ranges: Vec<Range<u32>>,

    // Indexed by instancerange
    pub instance_ranges: Vec<Range<u32>>,
    pub instance_range_mesh_entry: Vec<u32>,
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
