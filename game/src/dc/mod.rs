use std::collections::VecDeque;
use fate::math::Rgba;

#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    SetClearColor(Rgba<f32>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceContext {
    pub cmd_queue: VecDeque<Cmd>,
}

impl DeviceContext {
    pub fn new() -> Self {
        Self::with_capacity(512)
    }
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            cmd_queue: VecDeque::with_capacity(cap),
        }
    }
    fn push_cmd(&mut self, cmd: Cmd) {
        self.cmd_queue.push_back(cmd)
    }
    fn clear_cmds(&mut self) {
        self.cmd_queue.clear()
    }
    pub fn set_clear_color(&mut self, color: Rgba<f32>) {
        self.push_cmd(Cmd::SetClearColor(color))
    }
}

use system::*;

pub struct DeviceContextCommandClearerSystem;

impl DeviceContextCommandClearerSystem {
    pub fn new() -> Self { DeviceContextCommandClearerSystem }
}

impl System for DeviceContextCommandClearerSystem {
    fn draw(&mut self, g: &mut G, _: &Draw) {
        g.dc.clear_cmds();
    }
}