#![feature(panic_info_message)]
#![allow(unused_imports)]

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
extern crate gltf;
extern crate rand;
#[macro_use]
extern crate static_assertions;

// TODO list:
// - Load meshes (obj and GLTF)
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
// - Instanced rendering
// - Sound (OpenAL?)
// - WebGL port (porting to OpenGL ES 2, gonna hurt)

#[allow(unused_attributes)]
#[macro_export]
#[macro_use]
pub mod array_macro;

pub mod early;
pub mod platform;
pub mod main_game;
pub mod g;
pub mod quit;
pub mod frame_time;
pub mod event;
pub mod message;
pub mod system;
pub mod r_gl45;
pub mod input;
pub mod resources;
pub mod gpu;
pub mod gameplay;
pub mod mouse_cursor;
pub mod viewport;
pub mod cubemap;
pub mod texture2d;
pub mod mesh;
pub mod light;
pub mod material;
pub mod eid;
pub mod camera;
pub mod xform;

fn main() {
    early::setup_log();
    early::setup_panic_hook();
    early::setup_env();
    fate::main_loop::run(&mut main_game::MainGame::new())
}

