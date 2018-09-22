use fate::math::{Extent2, Rgba, Rect, Vec3, Vec4};
use fate::gx::{self, Object, gl::{self, types::*}};
use camera::{Camera, View};
use cubemap::CubemapSelector;

use super::gl_skybox::GLSkybox;
use super::gl_test_mdi_scene::GLTestMDIScene;

use gpu::GpuCmd;
use viewport::{ViewportVisitor, AcceptLeafViewport};
use cubemap::{CubemapArrayID};
use texture2d::Texture2DArrayID;
use mesh::VertexAttribIndex;
use system::*;

pub struct GLSystem {
    // Texture arrays
    cubemap_arrays: [GLuint; CubemapArrayID::MAX],
    texture2d_arrays: [GLuint; Texture2DArrayID::MAX],

    // Skybox
    skybox: GLSkybox,
    test_mdi_scene: GLTestMDIScene,
}

impl GLSystem {
    pub fn new() -> Self {
        let mut cubemap_arrays = [0; CubemapArrayID::MAX];
        let mut texture2d_arrays = [0; Texture2DArrayID::MAX];
        unsafe {
            gl::CreateTextures(gl::TEXTURE_CUBE_MAP_ARRAY, cubemap_arrays.len() as _, cubemap_arrays.as_mut_ptr());
            gl::CreateTextures(gl::TEXTURE_2D_ARRAY, texture2d_arrays.len() as _, texture2d_arrays.as_mut_ptr());
        }

        Self {
            cubemap_arrays,
            texture2d_arrays,
            skybox: GLSkybox::new(),
            test_mdi_scene: GLTestMDIScene::new(),
        }
    }
    pub fn cubemap_array(&self, id: CubemapArrayID) -> GLuint { self.cubemap_arrays[id.0 as usize] }
    pub fn texture2d_array(&self, id: Texture2DArrayID) -> GLuint { self.texture2d_arrays[id.0 as usize] }
    pub fn cubemap_array_mut(&mut self, id: CubemapArrayID) -> &mut GLuint { &mut self.cubemap_arrays[id.0 as usize] }
    pub fn texture2d_array_mut(&mut self, id: Texture2DArrayID) -> &mut GLuint { &mut self.texture2d_arrays[id.0 as usize] }
}

impl Drop for GLSystem {
    fn drop(&mut self) {
        let &mut Self {
            ref mut cubemap_arrays,
            ref mut texture2d_arrays,
            ..
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

        g.visit_viewports(&mut GLViewportVisitor { g, sys: self, });
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
                    gl::TextureStorage3D(self.texture2d_array(id), info.nb_levels as _, info.internal_format as _, info.size.w as _, info.size.h as _, info.nb_slots as _);
                },
                GpuCmd::CubemapArrayCreate(id) => {
                    let info = g.cubemap_array_info(id).unwrap();
                    gl::TextureStorage3D(self.cubemap_array(id), info.nb_levels as _, info.internal_format as _, info.size.w as _, info.size.h as _, (info.nb_cubemaps * 6) as _);
                },

                GpuCmd::Texture2DArrayDelete(id) => {
                    let tex = self.texture2d_array_mut(id);
                    gl::DeleteTextures(1, tex);
                    gl::CreateTextures(gl::TEXTURE_2D_ARRAY, 1, tex);
                },
                GpuCmd::CubemapArrayDelete(id) => {
                    let tex = self.cubemap_array_mut(id);
                    gl::DeleteTextures(1, tex);
                    gl::CreateTextures(gl::TEXTURE_CUBE_MAP_ARRAY, 1, tex);
                },


                GpuCmd::Texture2DArrayClear(id, level, color) => {
                    let color: Rgba<f32> = color; // Assert that we're dealing with the correct type
                    gl::ClearTexImage(self.texture2d_array(id), level as _, gl::RGBA, gl::FLOAT, color.as_ptr() as _);
                },
                GpuCmd::CubemapArrayClear(id, level, color) => {
                    let color: Rgba<f32> = color; // Assert that we're dealing with the correct type
                    gl::ClearTexImage(self.cubemap_array(id), level as _, gl::RGBA, gl::FLOAT, color.as_ptr() as _);
                },

                GpuCmd::Texture2DArraySubImage2D(id, slot, ref img) => {
                    let z = slot;
                    let depth = 1;
                    gl::TextureSubImage3D(self.texture2d_array(id), img.level as _, img.x as _, img.y as _, z as _, img.w as _, img.h as _, depth, img.format as _, img.type_ as _, img.data.as_ptr() as _);
                },
                GpuCmd::CubemapArraySubImage2D(id, slot, face, ref img) => {
                    let z = slot * 6 + face as usize;
                    let depth = 1;
                    gl::TextureSubImage3D(self.cubemap_array(id), img.level as _, img.x as _, img.y as _, z as _, img.w as _, img.h as _, depth, img.format as _, img.type_ as _, img.data.as_ptr() as _);
                },

                GpuCmd::CubemapArraySetMinFilter(id, filter)   => gl::TextureParameteri(self.cubemap_array(id), gl::TEXTURE_MIN_FILTER, filter as _),
                GpuCmd::CubemapArraySetMagFilter(id, filter)   => gl::TextureParameteri(self.cubemap_array(id), gl::TEXTURE_MAG_FILTER, filter as _),
                GpuCmd::Texture2DArraySetMinFilter(id, filter) => gl::TextureParameteri(self.texture2d_array(id), gl::TEXTURE_MIN_FILTER, filter as _),
                GpuCmd::Texture2DArraySetMagFilter(id, filter) => gl::TextureParameteri(self.texture2d_array(id), gl::TEXTURE_MAG_FILTER, filter as _),
            }
        }
    }
}

struct GLViewportVisitor<'a> {
    pub g: &'a G,
    pub sys: &'a GLSystem,
}

impl<'a> ViewportVisitor for GLViewportVisitor<'a> {
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

            let eid = args.info.camera;
            let view = View {
                xform: *self.g.eid_xform(eid).unwrap(),
                camera: *self.g.eid_camera(eid).unwrap(),
                viewport: Rect { x, y, w, h },
            };

            self.sys.test_mdi_scene.draw(&view);

            if let Some(skybox_cubemap_selector) = args.info.skybox_cubemap_selector {
                self.sys.skybox.draw(skybox_cubemap_selector, self.sys.cubemap_array(skybox_cubemap_selector.array_id), &view);
            }

            gl::Disable(gl::SCISSOR_TEST);
        }
    }
}
