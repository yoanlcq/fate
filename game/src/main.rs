extern crate fate;
extern crate fate_gx as gx;
extern crate dmc;
extern crate sdl2;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate backtrace;

// TODO list:
// - Passer au Core profile :/
// - Program: set_mat4(b"name\0", &m) -> Result<()> (grab all uniforms procedurally into a shader)
// - Cameras + move with mouse+arrows;
// - More stock meshes (e.g cube_smooth, cube, uv_sphere...)
// - More stock textures (e.g black, white, magenta (debug), checker ....)
// - Load textures (PNG, JPG, compressed...)
// - Skybox
// - Materials & pipelines
//   - Basic lighting
//   - PBR lighting
// - Text
// - Debug draw (using SceneCmds. Draw texture, draw text, draw debug mesh.....)
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

fn main() {
    early::setup_log();
    early::setup_panic_hook();
    early::setup_env();
    fate::main_loop::run(&mut game::Game::new())
}

