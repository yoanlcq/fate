use fate::math::Rgba;
use super::DC;

#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    SetClearColor(Rgba<f32>),
}

impl DC {
    pub fn set_clear_color(&mut self, color: Rgba<f32>) {
        self.push_cmd(Cmd::SetClearColor(color))
    }
}