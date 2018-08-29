use system::*;

#[derive(Debug)]
pub struct GpuEndFrame;

impl GpuEndFrame {
    pub fn new() -> Self { GpuEndFrame }
}

impl System for GpuEndFrame {
    fn draw(&mut self, g: &mut G, _: &Draw) {
        g.gpu_cmd_queue_clear();
    }
}


