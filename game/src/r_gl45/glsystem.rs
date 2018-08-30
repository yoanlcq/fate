use fate::math::{Extent2, Rgba, Rect};
use fate::gx::gl;

use gpu::GpuCmd;
use viewport::{ViewportVisitor, ViewportNodeID, ViewportNode, ViewportInfo, Split, SplitDirection, SplitOrigin, SplitUnit};
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
    fn draw(&mut self, g: &mut G, d: &Draw) {
        self.process_gpu_cmd_queue(g);

        let Extent2 { w, h } = g.input.canvas_size();
        unsafe {
            gl::Viewport(0, 0, w as _, h as _);
            let Rgba { r, g, b, a } = g.viewport_db().viewport_border_color();
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
            }
        }
    }
}

impl ViewportVisitor for GLSystem {
    fn accept_viewport(&mut self, id: ViewportNodeID, rect: Rect<u32, u32>, info: &mut ViewportInfo, parent: Option<ViewportNodeID>, border_px: u32) {
        unsafe {
            let Rect { x, y, w, h } = rect;
            gl::Viewport(x as _, y as _, w as _, h as _);

            // Temporary
            gl::Enable(gl::SCISSOR_TEST);

            let (bx, by) = (border_px, border_px);
            if w < bx+bx || h < by+by {
                return;
            }
            let (x, y, w, h) = (x+bx, y+by, w-bx-bx, h-by-by);
            let Rgba { r, g, b, a } = info.clear_color;
            gl::Scissor(x as _, y as _, w as _, h as _);
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT/* | gl::DEPTH_BUFFER_BIT*/);

            gl::Disable(gl::SCISSOR_TEST);
        }
    }
}
