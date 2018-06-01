#[macro_use] #[allow(unused_imports)]
extern crate static_assertions;
#[macro_use] #[allow(unused_imports)]
extern crate pretty_assertions;
extern crate vek;
extern crate sdl2;
extern crate gl;
extern crate alto;
extern crate freetype_sys;
#[macro_use] #[allow(unused_imports)]
extern crate serde;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate backtrace;

pub mod early;
pub mod duration_ext;
pub mod time;
pub mod game;
pub mod platform;
pub mod input;
pub mod v;
pub mod system;
pub mod grx;
pub mod gx;
pub mod mesh;
pub mod editor;
pub mod gameplay;
pub mod scene;
pub mod shape;
pub mod camera;
pub mod xform;
pub mod message;
pub mod paths;
pub mod font;
pub mod text;

use std::time::Duration;
use game::Game;
use time::{TimeManager, FpsCounter};

fn main() {
    early::setup_panic_hook();
    early::setup_env();
    early::setup_log();

    let g = Game::new("Grisui - Prelude", 800, 480);
    let mut time = TimeManager::with_fixed_dt_and_frame_time_ceil(
        Duration::from_millis(50),
        Duration::from_millis(250),
    );
    let mut fps_counter = FpsCounter::with_interval(Duration::from_millis(2000));
    let desired_max_fps = 64_f64;
    let enable_fixing_broken_vsync = true;

    'running: loop {
        time.begin_main_loop_iteration();

        time.pump_physics_steps(|t, dt| {
            g.pump_events();
            g.tick(t, dt);
        });

        if g.should_quit() {
            break 'running;
        }

        g.pump_events();
        g.draw(time.gfx_lerp_factor());
    
        if g.should_quit() {
            break 'running;
        }

        fps_counter.add_frame();
        if let Some(stats) = fps_counter.try_sampling_fps() {
            trace!("Main: New FPS stats: {}", &stats);
            if stats.fps() > desired_max_fps && enable_fixing_broken_vsync {
                time.fps_ceil = Some(desired_max_fps);
                warn!("Main: Broken VSync detected; FPS ceil is now set to {}", time.fps_ceil.unwrap());
            }
        }

        time.end_main_loop_iteration();
    }
}

