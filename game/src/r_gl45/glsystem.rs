use fate::math::{Extent2, Rgba, Rect};
use fate::gx::gl::{self, types::*};

use gpu::GpuCmd;
use viewport::{ViewportVisitor, AcceptLeafViewport, AcceptSplitViewport};
use cubemap::{CubemapArrayID};
use texture2d::Texture2DArrayID;
use system::*;


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
    fn accept_split_viewport(&mut self, _args: AcceptSplitViewport) {
    }
}
