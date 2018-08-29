use system::*;

#[derive(Debug)]
pub struct GpuCmdQueueClearer;

impl GpuCmdQueueClearer {
    pub fn new() -> Self { GpuCmdQueueClearer }
}

impl System for GpuCmdQueueClearer {
    fn draw(&mut self, g: &mut G, _: &Draw) {
        g.gpu_cmd_queue_clear();
    }
}


