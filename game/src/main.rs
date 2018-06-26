extern crate fate;
extern crate fate_gx as gx;
extern crate dmc;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate backtrace;

use std::time::{Duration, Instant};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::mem;
use fate::main_loop::{self, MainSystem, Tick, Draw};
use fate::lab::fps::{FpsManager, FpsCounter};
use fate::vek;
use vek::{Vec2, Extent2, Vec3, Rgba, Mat4};
use gx::{Object, gl::{self, types::*}};

mod early;

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

#[derive(Debug)]
struct FrameTimeManager {
    previous_frame_times: VecDeque<Duration>,
    current_frame_start: Option<Instant>,
    max_len: usize,
    average_frame_time: Duration,
    frame_time: Duration,
}

impl FrameTimeManager {
    pub fn with_max_len(max_len: usize) -> Self {
        Self {
            previous_frame_times: VecDeque::new(),
            current_frame_start: None,
            max_len,
            average_frame_time: Duration::default(),
            frame_time: Duration::default(),
        }
    }
    pub fn begin_main_loop_iteration(&mut self) {
        self.current_frame_start = Some(Instant::now());
    }
    pub fn end_main_loop_iteration  (&mut self) {
        self.previous_frame_times.push_back(self.current_frame_start.unwrap().elapsed());
        while self.previous_frame_times.len() > self.max_len {
            self.previous_frame_times.pop_front();
        }
        // Recompute average
        self.average_frame_time = {
            let mut sum = Duration::default();
            for d in self.previous_frame_times.iter() {
                sum += *d;
            }
            sum / self.previous_frame_times.len() as u32
        };
    }
    pub fn dt(&self) -> Duration {
        self.frame_time
    }
    pub fn smooth_dt(&self) -> Duration {
        self.average_frame_time
    }
}

#[derive(Debug)]
struct SharedGame {
    t: Duration, // Total physics time since the game started (accumulation of per-tick delta times)
    frame_time_manager: FrameTimeManager,
    pending_messages: VecDeque<Message>,
    scene: Scene,
}
pub type G = SharedGame;

impl SharedGame {
    pub fn new() -> Self {
        Self {
            t: Duration::default(),
            frame_time_manager: FrameTimeManager::with_max_len(60),
            pending_messages: VecDeque::new(),
            scene: Scene::new(),
        }
    }
    pub fn push_message(&mut self, msg: Message) {
        self.pending_messages.push_back(msg);
    }
}

mod event {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Event {
        Quit,
        MouseMotion(i32, i32),
        CanvasResized(u32, u32),
        // Imagine, many other different kinds of event
    }

    impl Event {
        pub fn dispatch(&self, sys: &mut System, g: &mut G) {
            match *self {
                Event::Quit => sys.on_quit(g),
                Event::MouseMotion(x, y) => sys.on_mouse_motion(g, (x, y)),
                Event::CanvasResized(w, h) => sys.on_canvas_resized(g, (w, h)),
            }
        }
    }
}
use self::event::Event;


#[derive(Debug)]
pub enum Message {
    Foo,
    Bar,
}


// All items take &mut self since we know we're single-threaded.
trait System {
    fn quit(&self) -> Quit { Quit::DontCare }
    fn begin_main_loop_iteration(&mut self, _g: &mut G) {}
    fn end_main_loop_iteration  (&mut self, _g: &mut G) {}
    fn tick(&mut self, _g: &mut G, _t: &Tick) {}
    fn draw(&mut self, _g: &mut G, _d: &Draw) {}

    // messages
    fn on_message(&mut self, _g: &mut G, _msg: &Message) {}

    // events
    fn on_quit(&mut self, _g: &mut G) {}
    fn on_mouse_motion(&mut self, _g: &mut G, _pos: (i32, i32)) {}
    fn on_mouse_button(&mut self, _g: &mut G, _btn: u32, _is_down: bool) {}
    fn on_canvas_resized(&mut self, _g: &mut G, _size: (u32, u32)) {}
}


// --- Systems

struct ExampleSystem;
impl System for ExampleSystem {}

#[derive(Debug, Default)]
struct Quitter(Quit);
impl System for Quitter {
    fn quit(&self) -> Quit { self.0 }
    fn on_quit(&mut self, _: &mut G) { self.0 = Quit::ShouldQuit; }
}

#[derive(Debug, Default)]
struct ParticleSystemsState {
    pub positions: Vec<(f32, f32)>,
}
impl ParticleSystemsState {
    pub fn replace_by_lerp(&mut self, a: &Self, b: &Self, t: f32) {
    }
}
struct ParticleSystemsManager {
    states: [ParticleSystemsState; 2], // Doesn't have to be 2
    gfx_state: ParticleSystemsState, // Similarly, we coule have several ones.
    index_of_prev_state: usize,
    index_of_next_state: usize,
}

impl ParticleSystemsManager {
    pub fn new() -> Self {
        Self {
            states: [ParticleSystemsState::default(), ParticleSystemsState::default() ],
            gfx_state: ParticleSystemsState::default(),
            index_of_prev_state: 0,
            index_of_next_state: 1,
        }
    }
}
impl System for ParticleSystemsManager {
    fn tick(&mut self, _g: &mut G, _t: &Tick) {
        self.index_of_next_state += 1;
        self.index_of_next_state %= self.states.len();
        self.index_of_prev_state += 1;
        self.index_of_prev_state %= self.states.len();
    }
    fn draw(&mut self, g: &mut G, d: &Draw) {
        self.gfx_state.replace_by_lerp(&self.states[self.index_of_prev_state], &self.states[self.index_of_next_state], d.tick_progress as _);
    }
}


// TODO: Add systems at runtime
// Solved: Just don't; Know your systems ahead of time! Selectively enable/disable them at runtime.
// TODO: Retrieve a specific system at runtime
// Solved: It depends. Finding by key is annoying; Why not directly typing g.my_sys ? We know our game.

fn gl_debug_message_callback(msg: &gx::DebugMessage) {
    match ::std::ffi::CString::new(msg.text) {
        Ok(cstr) => debug!("GL: {}", cstr.to_string_lossy()),
        Err(e) => debug!("GL (UTF-8 error): {}", e),
    };
}

const ATTRIB_POSITION_VEC3F32: GLuint = 0;
const ATTRIB_COLOR_RGBAF32: GLuint = 1;

static VS_SRC: &'static [u8] = b"
#version 450

uniform mat4 u_mvp;

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec4 a_color;

out vec4 v_color;

void main() {
    v_color = a_color;
    gl_Position = u_mvp * vec4(a_position, 1.0);
}
";
static FS_SRC: &'static [u8] = b"
#version 450

in vec4 v_color;

out vec4 f_color;

void main() {
    f_color = v_color;
}
";

#[derive(Debug)]
struct GLColorProgram {
    prog: gx::Program,
    u_mvp: GLint,
}

impl GLColorProgram {
    pub fn new() -> Result<Self, String> {
        let vs = gx::VertexShader::try_from_source(VS_SRC)?;
        let fs = gx::FragmentShader::try_from_source(FS_SRC)?;
        let prog = gx::Program::try_from_vert_frag(&vs, &fs)?;
        let u_mvp = unsafe {
            gl::GetUniformLocation(prog.gl_id(), b"u_mvp\0".as_ptr() as _)
        };
        if u_mvp == -1 {
            return Err(format!("u_mvp is invalid!"));
        }

        Ok(Self { prog, u_mvp, })
    }
    pub fn set_u_mvp(&self, m: &Mat4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(self.u_mvp, 1, m.gl_should_transpose() as _, &m[(0, 0)]);
        }
    }
    pub fn gl_id(&self) -> GLuint {
        self.prog.gl_id()
    }
}

#[derive(Debug)]
struct Mesh {
    pub topology: GLenum,
    pub vposition: Vec<Vec3<f32>>, // Not optional
    pub vcolor: Vec<Rgba<f32>>, // Optional. If there's only one element, it is used for all vertices.
    pub indices: Vec<u16>, // Optional. If empty, it's rendered using glDrawArrays.
}

impl Mesh {
    pub fn new_cube() -> Self {
        let vposition: [Vec3<f32>; 14] = [
            Vec3::new(-1.,  1.,  1.), // Front-top-left
            Vec3::new( 1.,  1.,  1.), // Front-top-right
            Vec3::new(-1., -1.,  1.), // Front-bottom-left
            Vec3::new( 1., -1.,  1.), // Front-bottom-right
            Vec3::new( 1., -1., -1.), // Back-bottom-right
            Vec3::new( 1.,  1.,  1.), // Front-top-right
            Vec3::new( 1.,  1., -1.), // Back-top-right
            Vec3::new(-1.,  1.,  1.), // Front-top-left
            Vec3::new(-1.,  1., -1.), // Back-top-left
            Vec3::new(-1., -1.,  1.), // Front-bottom-left
            Vec3::new(-1., -1., -1.), // Back-bottom-left
            Vec3::new( 1., -1., -1.), // Back-bottom-right
            Vec3::new(-1.,  1., -1.), // Back-top-left
            Vec3::new( 1.,  1., -1.), // Back-top-right
        ];

        Self {
            topology: gl::TRIANGLE_STRIP,
            vposition: vposition.to_vec(),
            vcolor: vec![Rgba::red()],
            indices: vec![],
        }
    }
}

pub type MeshID = u32;

#[derive(Debug)]
enum SceneCommand {
    MeshUpdated { mesh_id: MeshID }
}

#[derive(Debug)]
struct Scene {
    pub meshes: HashMap<MeshID, Mesh>,
    pub command_queue: VecDeque<SceneCommand>,
}

impl Scene {
    pub fn new() -> Self {
        let cube_mesh_id = 1;
        let mut meshes = HashMap::new();
        let mut command_queue = VecDeque::new();

        meshes.insert(cube_mesh_id, Mesh::new_cube());
        command_queue.push_back(SceneCommand::MeshUpdated { mesh_id: cube_mesh_id });

        Self {
            meshes,
            command_queue,
        }
    }
}

fn gx_buffer_data<T>(target: gx::BufferTarget, data: &[T], usage: gx::BufferUsage) {
    unsafe {
        gl::BufferData(target as _, mem::size_of_val(data) as _, data.as_ptr() as _, usage as _);
    }
}
fn gx_buffer_data_dsa<T>(buf: &gx::Buffer, data: &[T], usage: gx::BufferUsage) {
    unsafe {
        gl::BindBuffer(gx::BufferTarget::Array as _, buf.gl_id());
        gx_buffer_data(gx::BufferTarget::Array, data, usage);
        gl::BindBuffer(gx::BufferTarget::Array as _, 0);
    }
}


#[derive(Debug)]
struct GLSystem {
    pub prog: GLColorProgram,
    pub mesh_position_buffers: HashMap<MeshID, gx::Buffer>,
    pub mesh_color_buffers: HashMap<MeshID, gx::Buffer>,
    pub mesh_index_buffers: HashMap<MeshID, gx::Buffer>,
}

impl GLSystem {
    pub fn new() -> Self {
        Self {
            prog: GLColorProgram::new().unwrap(),
            mesh_position_buffers: Default::default(),
            mesh_color_buffers: Default::default(),
            mesh_index_buffers: Default::default(),
        }
    }

    fn render_scene(&mut self, scene: &Scene, d: &Draw) {
        unsafe {
            gl::UseProgram(self.prog.gl_id());
        }
        for (mesh_id, mesh) in scene.meshes.iter() {

            self.prog.set_u_mvp(&Mat4::default());

            if let Some(idx_buffer) = self.mesh_index_buffers.get(mesh_id) {
                unimplemented!("Index buffers are not supported yet");
            }

            assert!(!mesh.vposition.is_empty());
            let pos_buffer = self.mesh_position_buffers.get(mesh_id).expect("Meshes must have a position buffer (for now)!");
            unsafe {
                gl::BindBuffer(gx::BufferTarget::Array as _, pos_buffer.gl_id());
                gl::EnableVertexAttribArray(ATTRIB_POSITION_VEC3F32);
                gl::VertexAttribPointer(ATTRIB_POSITION_VEC3F32, 3, gl::FLOAT, gl::FALSE, 3*4, 0 as _);
                gl::BindBuffer(gx::BufferTarget::Array as _, 0);
            }

            let set_default_color = |rgba: Rgba<f32>| unsafe {
                gl::DisableVertexAttribArray(ATTRIB_COLOR_RGBAF32);
                gl::VertexAttrib4f(ATTRIB_COLOR_RGBAF32, rgba.r, rgba.g, rgba.b, rgba.a);
            };
            match self.mesh_color_buffers.get(mesh_id) {
                None => set_default_color(Rgba::white()),
                Some(col_buffer) => {
                    match mesh.vcolor.len() {
                        0 => set_default_color(Rgba::white()),
                        1 => set_default_color(mesh.vcolor[0]),
                        _ => unsafe {
                            gl::BindBuffer(gx::BufferTarget::Array as _, col_buffer.gl_id());
                            gl::EnableVertexAttribArray(ATTRIB_COLOR_RGBAF32);
                            gl::VertexAttribPointer(ATTRIB_COLOR_RGBAF32, 4, gl::FLOAT, gl::FALSE, 4*4, 0 as _);
                            gl::BindBuffer(gx::BufferTarget::Array as _, 0);
                        },
                    }
                },
            }

            unsafe {
                gl::DrawArrays(mesh.topology, 0, mesh.vposition.len() as _);
            }
        }
    }
    fn pump_scene_commands(&mut self, scene: &mut Scene) {
        while let Some(cmd) = scene.command_queue.pop_front() {
            self.handle_scene_command(scene, &cmd);
        }
    }
    fn handle_scene_command(&mut self, scene: &Scene, cmd: &SceneCommand) {
        match *cmd {
            SceneCommand::MeshUpdated { mesh_id } => {
                if let Some(&Mesh { topology: _, ref vposition, ref vcolor, ref indices, }) = scene.meshes.get(&mesh_id) {
                    gx_buffer_data_dsa(self.mesh_position_buffers.entry(mesh_id).or_insert(gx::Buffer::new()), vposition, gx::BufferUsage::StaticDraw);
                    if vcolor.is_empty() {
                        self.mesh_color_buffers.remove(&mesh_id);
                    } else {
                        gx_buffer_data_dsa(self.mesh_color_buffers.entry(mesh_id).or_insert(gx::Buffer::new()), vcolor, gx::BufferUsage::StaticDraw);
                    }
                    if indices.is_empty() {
                        self.mesh_index_buffers.remove(&mesh_id);
                    } else {
                        gx_buffer_data_dsa(self.mesh_index_buffers.entry(mesh_id).or_insert(gx::Buffer::new()), indices, gx::BufferUsage::StaticDraw);
                    }
                }
            },
        }
    }
}

impl System for GLSystem {
    fn on_canvas_resized(&mut self, g: &mut G, size: (u32, u32)) {
        debug!("GL: Setting viewport to (0, 0, {}, {})", size.0, size.1);
        unsafe {
            gl::Viewport(0, 0, size.0 as _, size.1 as _);
        }
    }
    fn draw(&mut self, g: &mut G, d: &Draw) {
        let scene = &mut g.scene;
        self.pump_scene_commands(scene);
        self.render_scene(scene, d);
    }
}

struct Game {
    dmc: dmc::Context,
    window: dmc::Window,
    gl_context: dmc::gl::GLContext,
    shared: RefCell<SharedGame>,
    systems: Vec<Box<System>>,
    fps_manager: FpsManager,
    fps_ceil: Option<f64>,
}

impl Game {
    pub fn new() -> Self {
        let gl_pixel_format_settings = dmc::gl::GLPixelFormatSettings {
            msaa: dmc::gl::GLMsaa { buffer_count: 1, sample_count: 4 },
            depth_bits: 24,
            stencil_bits: 8,
            double_buffer: true,
            stereo: false,
            red_bits: 8,
            green_bits: 8,
            blue_bits: 8,
            alpha_bits: 8,
            accum_red_bits: 0,
            accum_blue_bits: 0,
            accum_green_bits: 0,
            accum_alpha_bits: 0,
            aux_buffers: 0,
            transparent: false,
        };
        let gl_context_settings = dmc::gl::GLContextSettings {
            version: dmc::gl::GLVersion::new_desktop(4, 5),
            profile: dmc::gl::GLProfile::Compatibility,
            debug: true,
            forward_compatible: true,
            robust_access: None,
        };
        info!("Using GL pixel format settings: {:#?}", gl_pixel_format_settings);
        info!("Using GL context settings: {:#?}", gl_context_settings);
        let dmc = dmc::Context::new().unwrap();
        let window = dmc.create_window(&dmc::WindowSettings {
            high_dpi: false,
            opengl: Some(&dmc::gl::GLDefaultPixelFormatChooser::from(&gl_pixel_format_settings)),
        }).unwrap();
        let gl_context = window.create_gl_context(&gl_context_settings).unwrap();
        window.make_gl_context_current(Some(&gl_context)).unwrap();
        if let Err(_) = window.gl_set_swap_interval(dmc::gl::GLSwapInterval::LateSwapTearing) {
            let _ = window.gl_set_swap_interval(dmc::gl::GLSwapInterval::VSync);
        }
        gl::load_with(|s| {
            let f = gl_context.proc_address(s);
            trace!("GL: {}: {}", if f.is_null() { "Failed" } else { "Loaded" }, s);
            f
        });
        info!("OpenGL context summary:\n{}", gx::ContextSummary::new());
        gx::boot_gl();
        gx::set_debug_message_callback(Some(gl_debug_message_callback));
        gx::log_debug_message("OpenGL debug logging is enabled.");

        let shared = SharedGame::new();
        let mut systems = Vec::new();
        systems.push(Box::new(ExampleSystem) as Box<System>);
        systems.push(Box::new(Quitter::default()));
        systems.push(Box::new(GLSystem::new()));
        systems.push(Box::new(ParticleSystemsManager::new()));
        let fps_manager = FpsManager {
            fps_counter: FpsCounter::with_interval(Duration::from_secs(1)),
            desired_fps_ceil: 64.,
            enable_fixing_broken_vsync: true,
        };
 
        window.set_size(Extent2::new(800, 600)).unwrap();
        window.set_title("Test Game").unwrap();
        window.show().unwrap();

        Self {
            dmc,
            window,
            gl_context,
            shared: RefCell::new(shared),
            systems,
            fps_manager,
            fps_ceil: None,
        }
    }
    pub fn poll_event(&mut self) -> Option<Event> {
        use dmc::Event as DmcEvent;
        match self.dmc.poll_event()? {
            DmcEvent::Quit => Some(Event::Quit),
            DmcEvent::WindowCloseRequested { .. } => Some(Event::Quit),
            DmcEvent::MouseMotion { position: Vec2 { x, y }, .. } => Some(Event::MouseMotion(x as _, y as _)),
            DmcEvent::WindowResized { size: Extent2 { w, h }, .. } => Some(Event::CanvasResized(w, h)),
            _ => None,
        }
    }
    pub fn pump_messages(&mut self) {
        while let Some(msg) = self.shared.borrow_mut().pending_messages.pop_front() {
            for sys in self.systems.iter_mut() {
                sys.on_message(&mut self.shared.borrow_mut(), &msg);
            }
        }
    }
}
impl MainSystem for Game {
    fn quit(&self) -> bool {
        let mut should_quit = 0;
        let mut dont_quit = 0;
        for sys in self.systems.iter() {
            match sys.quit() {
                Quit::ForceQuit => return true,
                Quit::ShouldQuit => should_quit += 1,
                Quit::DontQuit => dont_quit += 1,
                Quit::DontCare => (),
            };
        }
        should_quit > 0 && dont_quit == 0
    }

    fn fps_ceil(&self) -> Option<f64> { self.fps_ceil }
    fn tick_dt(&self) -> Duration { Duration::from_millis(16) }
    fn frame_time_ceil(&self) -> Duration { Duration::from_millis(250) }

    fn begin_main_loop_iteration(&mut self) {
        self.shared.borrow_mut().frame_time_manager.begin_main_loop_iteration();
        for sys in self.systems.iter_mut() {
            sys.begin_main_loop_iteration(&mut self.shared.borrow_mut());
        }
    }
    fn end_main_loop_iteration  (&mut self) {
        for sys in self.systems.iter_mut() {
            sys.end_main_loop_iteration(&mut self.shared.borrow_mut());
        }
        self.shared.borrow_mut().frame_time_manager.end_main_loop_iteration();
        let fps_stats = self.fps_manager.end_main_loop_iteration(&mut self.fps_ceil);
        if let Some(fps_stats) = fps_stats {
            println!("{}", fps_stats);
        }
    }
    fn pump_events(&mut self) {
        self.pump_messages();
        while let Some(ev) = self.poll_event() {
            for sys in self.systems.iter_mut() {
                ev.dispatch(sys.as_mut(), &mut self.shared.borrow_mut());
            }
            self.pump_messages();
        }
    } 
    fn tick(&mut self, tick: &Tick) {
        let mut shared = self.shared.borrow_mut();
        shared.t += tick.dt;
        for sys in self.systems.iter_mut() {
            sys.tick(&mut shared, tick);
        }
    }
    fn draw(&mut self, draw: &Draw) {
        unsafe {
            gl::ClearColor(1., 0., 1., 1.);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        for sys in self.systems.iter_mut() {
            sys.draw(&mut self.shared.borrow_mut(), draw);
        }
        self.window.gl_swap_buffers().unwrap();
    }
}

fn main() {
    early::setup_log();
    early::setup_panic_hook();
    early::setup_env();
    main_loop::run(&mut Game::new())
}

