use fate::math::{Extent2, Rgba, Rect};
use fate::gx::gl;

use gpu::GpuCmd;
use g::{ViewportNodeID, ViewportNode, ViewportInfo, Split, SplitDirection, SplitOrigin, SplitUnit};
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
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        self.draw_viewport(g, Rect { x: 0, y: 0, w, h }, g.root_viewport_node_id(), d);
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
    fn draw_viewport(&mut self, g: &G, rect: Rect<u32, u32>, id: ViewportNodeID, d: &Draw) {
        let node = g.viewport_node(id).unwrap();
        match *node {
            ViewportNode::Whole { ref info, .. } => unsafe {
                let Rect { x, y, w, h } = rect;
                let Rgba { r, g, b, a } = info.clear_color;

                gl::Viewport(x as _, y as _, w as _, h as _);

                // Temprary
                gl::Enable(gl::SCISSOR_TEST);
                gl::Scissor(x as _, y as _, w as _, h as _);
                gl::ClearColor(r, g, b, a);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::Disable(gl::SCISSOR_TEST);
            },
            ViewportNode::Split { children: (c0, c1), split: Split { origin, unit, value, direction }, .. } => {
                // assume value is relative to middle
                let mut r0 = rect;
                let mut r1 = rect;
                match direction {
                    SplitDirection::Horizontal => {
                        r0.h /= 2;
                        r1.h = rect.h - r0.h;
                        r1.y = rect.y + r0.h;
                    },
                    SplitDirection::Vertical => {
                        r0.w /= 2;
                        r1.w = rect.w - r0.w;
                        r1.x = rect.x + r0.w;
                    },
                }
                self.draw_viewport(g, r0, c0, d);
                self.draw_viewport(g, r1, c1, d);
            },
        }
    }
}
