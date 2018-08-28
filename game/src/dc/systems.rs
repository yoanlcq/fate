use system::*;

#[derive(Debug)]
pub struct DcCmdClearerSystem;

impl DcCmdClearerSystem {
    pub fn new() -> Self { DcCmdClearerSystem }
}

impl System for DcCmdClearerSystem {
    fn draw(&mut self, g: &mut G, _: &Draw) {
        g.dc.clear_cmds();
    }
}