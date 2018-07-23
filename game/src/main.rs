#[macro_use]
extern crate fate;
#[allow(unused_imports)]
#[macro_use]
extern crate dmc;
extern crate sdl2;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate backtrace;

// TODO list:
// - Text
// - ECS.
//   - Les meshs qui veulent tourner doivent opter pour un component
// - Materials & pipelines
//   - Basic lighting
//   - PBR lighting
// - Debug draw (using SceneCmds. Draw texture, draw text, draw debug mesh, draw wireframe, draw normals.....)
// - Hot async reloading of:
//   - Shaders
//   - Resources
// - GUI
// - Load meshes (obj and GLTF)
// - Instanced rendering

pub mod early;
pub mod platform;
pub mod game;
pub mod quit;
pub mod frame_time;
pub mod event;
pub mod message;
pub mod system;
pub mod gamegl;
pub mod scene;
pub mod input;
pub mod atlas;

fn main() {
    early::setup_log();
    early::setup_panic_hook();
    early::setup_env();
    fate::main_loop::run(&mut game::Game::new())
}

