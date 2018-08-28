use fate::math::{Extent2, Rgba};
use fate::gx::gl;
use system::*;
use dc;


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
    fn draw(&mut self, g: &mut G, _d: &Draw) {
        self.process_dc_cmds(&g.dc);
        self.gl_clear();
    }
}

impl GLSystem {
    fn process_dc_cmds(&mut self, dc: &dc::DC) {
        for cmd in dc.cmd_queue.iter() {
            self.execute_dc_cmd(cmd);
        }
    }
    fn execute_dc_cmd(&mut self, cmd: &dc::Cmd) {
        unsafe {
            match *cmd {
                dc::Cmd::SetClearColor(Rgba { r, g, b, a }) => gl::ClearColor(r, g, b, a),
            }
        }
    }
    fn gl_clear(&self) {
        unsafe {
            let Extent2 { w, h } = self.viewport_size;
            gl::Viewport(0, 0, w as _, h as _);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}
