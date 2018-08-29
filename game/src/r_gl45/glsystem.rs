use fate::math::{Extent2};
use fate::gx::gl;
use system::*;


#[derive(Debug)]
pub struct GLSystem {
    viewport_size: Extent2<u32>,
}

impl GLSystem {
    pub fn new(viewport_size: Extent2<u32>, _g: &G) -> Self {
        Self { viewport_size }
    }
}

impl System for GLSystem {
    fn on_canvas_resized(&mut self, _g: &mut G, size: Extent2<u32>) {
        self.viewport_size = size;
    }
    fn draw(&mut self, _g: &mut G, _d: &Draw) {
        self.gl_clear();
    }
}

impl GLSystem {
    fn gl_clear(&self) {
        unsafe {
            let Extent2 { w, h } = self.viewport_size;
            gl::Viewport(0, 0, w as _, h as _);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}
