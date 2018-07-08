extern crate fate;
#[allow(unused_imports)]
#[macro_use]
extern crate fate_gx as gx;
extern crate dmc;
extern crate sdl2;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate backtrace;

// TODO list:
// - X11: Bug du 'e' chapeau
// - Je veux orienter la caméra avec la souris
// - Je veux déplacer la caméra avec les flèches
// - ECS.
//   - La skybox utilise un autre shader
//   - La skybox n'est pas mise à jour
//   - Les meshs qui veulent tourner doivent opter pour un component
// - Text
// - More stock textures (e.g black, white, magenta (debug), checker ....)
// - Load textures (PNG, JPG, compressed...)
// - En fait je veux une vraie skybox
// - Materials & pipelines
//   - Basic lighting
//   - PBR lighting
// - Debug draw (using SceneCmds. Draw texture, draw text, draw debug mesh, draw wireframe, draw normals.....)
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

