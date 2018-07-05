use std::mem;
use std::collections::HashMap;
use fate::vek::{Rgba, Mat4, Extent2, Vec3, FrustumPlanes};
use gx::{self, Object, gl::{self, types::*}};
use scene::{Scene, MeshID, Mesh, MeshInstance, SceneCommand, CameraProjectionMode, Camera};
use system::*;

static mut NB_ERRORS: usize = 0;

pub fn gl_error_hook(e: Option<gx::Error>, s: &str) {
    match e {
        Some(e) => {
            error!("GL error: {:?} ({})", e, s);
            unsafe { NB_ERRORS += 1; }
        },
        None => if unsafe { NB_ERRORS > 0 } {
            panic!("Encountered {} OpenGL errors.", unsafe { NB_ERRORS });
        }
    }
}

pub fn gl_debug_message_callback(msg: &gx::DebugMessage) {
    match ::std::ffi::CString::new(msg.text) {
        Ok(cstr) => debug!("GL: {}", cstr.to_string_lossy()),
        Err(e) => debug!("GL (UTF-8 error): {}", e),
    };
}

const ATTRIB_POSITION_VEC3F32: GLuint = 0;
const ATTRIB_COLOR_RGBAF32: GLuint = 1;

static VS_SRC: &'static [u8] = b"
#version 450 core

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
#version 450 core

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
        check_gl!("GLColorProgram: Before any shader");
        let vs = gx::VertexShader::try_from_source(VS_SRC)?;
        let fs = gx::FragmentShader::try_from_source(FS_SRC)?;
        let prog = gx::Program::try_from_vert_frag(&vs, &fs)?;
        let u_mvp = unsafe {
            gl::GetUniformLocation(prog.gl_id(), b"u_mvp\0".as_ptr() as _)
        };
        if u_mvp == -1 {
            return Err(format!("u_mvp is invalid!"));
        }
        check_gl!("GLColorProgram");

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

    pub fn attribs(&self) -> Vec<AttribInfo> {
        let mut count = 0;
        unsafe {
            gl::GetProgramiv(self.gl_id(), gl::ACTIVE_ATTRIBUTES, &mut count);
        }
        for i in 0..count {
            let mut name = [0_u8; 256];
            let mut name_len = 0;
            let mut var_size = 0;
            let mut var_type = 0;
            unsafe {
                gl::GetActiveAttrib(self.gl_id(), i as _, name.len() as _, &mut name_len, &mut var_size, &mut var_type, name.as_mut_ptr() as _);
            }
        }
        unimplemented!{}
    }

    pub fn uniforms(&self) -> Vec<UniformInfo> {
        let mut count = 0;
        unsafe {
            gl::GetProgramiv(self.gl_id(), gl::ACTIVE_UNIFORMS, &mut count);
        }
        for i in 0..count {
            let mut name = [0_u8; 256];
            let mut name_len = 0;
            let mut var_size = 0;
            let mut var_type = 0;
            unsafe {
                gl::GetActiveUniform(self.gl_id(), i as _, name.len() as _, &mut name_len, &mut var_size, &mut var_type, name.as_mut_ptr() as _);
            }
        }
        unimplemented!{}
    }
}

#[derive(Debug)]
pub struct AttribInfo;

#[derive(Debug)]
pub struct UniformInfo;


fn gx_buffer_data<T>(target: gx::BufferTarget, data: &[T], usage: gx::BufferUsage) {
    unsafe {
        check_gl!(gl::BufferData(target as _, mem::size_of_val(data) as _, data.as_ptr() as _, usage as _));
    }
}
fn gx_buffer_data_dsa<T>(buf: &gx::Buffer, data: &[T], usage: gx::BufferUsage) {
    unsafe {
        check_gl!(gl::BindBuffer(gx::BufferTarget::Array as _, buf.gl_id()));
        gx_buffer_data(gx::BufferTarget::Array, data, usage);
        gl::BindBuffer(gx::BufferTarget::Array as _, 0);
    }
}


#[derive(Debug)]
pub struct GLSystem {
    viewport_size: Extent2<u32>,
    prog: GLColorProgram,
    mesh_position_buffers: HashMap<MeshID, gx::Buffer>,
    mesh_normal_buffers: HashMap<MeshID, gx::Buffer>,
    mesh_color_buffers: HashMap<MeshID, gx::Buffer>,
    mesh_index_buffers: HashMap<MeshID, gx::Buffer>,
    pipeline_statistics_arb_queries: HashMap<gx::QueryTarget, gx::Query>,
}

impl GLSystem {
    pub fn new(viewport_size: Extent2<u32>) -> Self {
        let pipeline_statistics_arb_targets = [
            gx::QueryTarget::VerticesSubmittedARB              ,
            gx::QueryTarget::PrimitivesSubmittedARB            ,
            gx::QueryTarget::VertexShaderInvocationsARB        ,
            gx::QueryTarget::TessControlShaderPatchesARB       ,
            gx::QueryTarget::TessEvaluationShaderInvocationsARB,
            gx::QueryTarget::GeometryShaderInvocations         ,
            gx::QueryTarget::GeometryShaderPrimitivesEmittedARB,
            gx::QueryTarget::FragmentShaderInvocationsARB      ,
            gx::QueryTarget::ComputeShaderInvocationsARB       ,
            gx::QueryTarget::ClippingInputPrimitivesARB        ,
            gx::QueryTarget::ClippingOutputPrimitivesARB       ,
        ];
        let pipeline_statistics_arb_queries = if pipeline_statistics_arb_targets[0].is_supported() {
            debug!("GL: ARB_pipeline_statistics_query is supported.");
            pipeline_statistics_arb_targets.into_iter()
                .map(|target| (*target, gx::Query::new()))
                .collect()
        } else {
            debug!("GL: ARB_pipeline_statistics_query is unsupported.");
            Default::default()
        };
        Self {
            viewport_size,
            prog: GLColorProgram::new().unwrap(),
            mesh_position_buffers: Default::default(),
            mesh_normal_buffers: Default::default(),
            mesh_color_buffers: Default::default(),
            mesh_index_buffers: Default::default(),
            pipeline_statistics_arb_queries,
        }
    }

    fn render_scene(&mut self, scene: &Scene, _draw: &Draw) {
        unsafe {
            check_gl!(gl::UseProgram(self.prog.gl_id()));
        }
        for camera in scene.cameras.values() {
            self.render_scene_with_camera(scene, _draw, camera);
        }
    }

    fn render_scene_with_camera(&mut self, scene: &Scene, _draw: &Draw, camera: &Camera) {
        let &Camera {
            position: camera_position,
            target: camera_target,
            scale: camera_scale,
            projection_mode,
            fov_y_radians: fov_y,
            near,
            far,
        } = camera;

        for &MeshInstance { ref mesh_id, xform } in scene.mesh_instances.values() {
            let mesh = &scene.meshes[mesh_id];

            let aspect_ratio = {
                let Extent2 { w, h } = self.viewport_size;
                assert_ne!(w, 0);
                assert_ne!(h, 0);
                w as f32 / h as f32
            };
            let proj = match projection_mode {
                CameraProjectionMode::Perspective => Mat4::perspective_lh_no(fov_y, aspect_ratio, near, far),
                CameraProjectionMode::Ortho => Mat4::orthographic_lh_no(FrustumPlanes {
                    right: aspect_ratio,
                    left: -aspect_ratio,
                    top: 1.,
                    bottom: -1.,
                    near,
                    far,
                }),
            };
            let view = Mat4::<f32>::scaling_3d(camera_scale.recip())
                * Mat4::look_at(camera_position, camera_target, Vec3::up());
            let model = Mat4::from(xform);
            let mvp = proj * view * model;

            self.prog.set_u_mvp(&mvp);
            check_gl!("Set u_mvp");

            /*
            unsafe {
                gl::Disable(gl::CULL_FACE);
                //gl::CullFace(gl::BACK);
            }
            */

            assert!(!mesh.vposition.is_empty());
            let pos_buffer = self.mesh_position_buffers.get(mesh_id).expect("Meshes must have a position buffer (for now)!");
            unsafe {
                check_gl!(gl::BindBuffer(gx::BufferTarget::Array as _, pos_buffer.gl_id()));
                check_gl!(gl::EnableVertexAttribArray(ATTRIB_POSITION_VEC3F32));
                check_gl!(gl::VertexAttribPointer(ATTRIB_POSITION_VEC3F32, 3, gl::FLOAT, gl::FALSE, 3*4, 0 as _));
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

            if let Some(idx_buffer) = self.mesh_index_buffers.get(mesh_id) {
                if !mesh.indices.is_empty() {
                    unsafe {
                        gl::BindBuffer(gx::BufferTarget::ElementArray as _, idx_buffer.gl_id());
                        assert!(mem::size_of_val(&mesh.indices[0]) == 2); // for gl::UNSIGNED_SHORT
                        gl::DrawElements(mesh.topology, mesh.indices.len() as _, gl::UNSIGNED_SHORT, 0 as _);
                        gl::BindBuffer(gx::BufferTarget::ElementArray as _, 0);
                    }
                }
            } else {
                unsafe {
                    gl::DrawArrays(mesh.topology, 0, mesh.vposition.len() as _);
                }
            }
        }
    }
    fn pump_scene_draw_commands(&mut self, scene: &mut Scene) {
        for cmd in scene.draw_commands_queue.iter() {
            self.handle_scene_command(scene, cmd);
        }
    }
    fn handle_scene_command(&mut self, scene: &Scene, cmd: &SceneCommand) {
        match *cmd {
            SceneCommand::AddMesh(mesh_id) => {
                if let Some(&Mesh { topology: _, ref vposition, ref vnormal, ref vcolor, ref indices, }) = scene.meshes.get(&mesh_id) {
                    gx_buffer_data_dsa(self.mesh_position_buffers.entry(mesh_id).or_insert(gx::Buffer::new()), vposition, gx::BufferUsage::StaticDraw);
                    gx_buffer_data_dsa(self.mesh_normal_buffers.entry(mesh_id).or_insert(gx::Buffer::new()), vnormal, gx::BufferUsage::StaticDraw);
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
            SceneCommand::AddMeshInstance(_id) => {},
        }
    }
}

impl System for GLSystem {
    fn on_canvas_resized(&mut self, _g: &mut G, size: Extent2<u32>) {
        self.viewport_size = size;
    }
    fn draw(&mut self, g: &mut G, d: &Draw) {
        unsafe {
            let Extent2 { w, h } = self.viewport_size;
            gl::Viewport(0, 0, w as _, h as _);
            gl::ClearColor(1., 0., 1., 1.);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            check_gl!("Viewport + clear");
        }

        for (target, query) in self.pipeline_statistics_arb_queries.iter() {
            target.begin(query);
        }

        let scene = &mut g.scene;
        self.pump_scene_draw_commands(scene);
        self.render_scene(scene, d);

        for target in self.pipeline_statistics_arb_queries.keys() {
            target.end();
        }
        // FIXME: No, we don't wanna wait!!
        for (target, query) in self.pipeline_statistics_arb_queries.iter() {
            let result = query.wait_result();
            info!("Pipeline statistics ARB: {:?} = {}", target, result);
        }
    }
}

