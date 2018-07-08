use system::*;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Quit {
    DontCare,
    DontQuit,
    ShouldQuit,
    ForceQuit,
}

impl Default for Quit {
    fn default() -> Self {
        Quit::DontCare
    }
}

#[derive(Debug, Default)]
pub struct Quitter(Quit);

impl System for Quitter {
    fn quit(&self) -> Quit { self.0 }
    fn on_quit(&mut self, _: &mut G) { self.0 = Quit::ShouldQuit; }
}
