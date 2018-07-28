#![feature(panic_info_message)]

#[macro_use]
extern crate fate;
#[allow(unused_imports)]
#[macro_use]
extern crate dmc;
extern crate sdl2;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate backtrace;

// TODO list:
// - Debug (immediate) draw (commands pushed to Scene);
//   - PushMatrix/PopMatrix;
//   - 2D:
//     - Text;
//     - Gradiant quads;
//   - 3D:
//     - Lines, lines strips;
//     - Triangles;
// - Async load:
//   - Resources;
// - ECS.
//   - Les meshs qui veulent tourner doivent opter pour un component
// - Materials & pipelines
//   - Basic lighting
//   - PBR lighting
// - Debug draw (using SceneCmds. Draw texture, draw text, draw debug mesh, draw wireframe, draw normals.....)
// - Hot reloading:
//   - Shaders
//   - Resources
// - GUI
// - Load meshes (obj and GLTF)
// - Instanced rendering
// - Sound (OpenAL?)
// - WebGL port (porting to OpenGL ES 2, gonna hurt)

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
pub mod resources;
pub mod async;

fn main() {
    early::setup_log();
    early::setup_panic_hook();
    early::setup_env();
    fate::main_loop::run(&mut game::Game::new())
}

