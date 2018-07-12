use std::mem;
use std::collections::HashMap;
use fate::vek::{Rgba, Rgb, Mat4, Extent2, Vec3, FrustumPlanes};
use gx::{self, Object, gl};
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

#[repr(u32)]
pub enum VAttrib {
    PositionVec4f32 = 0,
    NormalVec4f32 = 1,
    TangentVec4f32 = 2,
    BiTangentVec4f32 = 3,
    ColorRgbau8 = 4,
}

static VS_SRC: &'static [u8] = b"
#version 450 core

uniform mat4 u_proj_matrix;
uniform mat4 u_modelview_matrix;
uniform mat4 u_normal_matrix;

layout(location = 0) in vec4 a_position;
layout(location = 1) in vec4 a_normal;
layout(location = 4) in vec4 a_color;

out vec4 v_position_viewspace;
out vec4 v_normal_viewspace;
out vec4 v_color;

void main() {
    v_position_viewspace = u_modelview_matrix * vec4(a_position.xyz, 1.0);
    v_normal_viewspace = u_normal_matrix * vec4(a_normal.xyz, 0.0);
    v_color = a_color;
    gl_Position = u_proj_matrix * v_position_viewspace;
}
";
static FS_SRC: &'static [u8] = b"
#version 450 core

uniform vec3 u_light_position_viewspace;
uniform vec3 u_light_color;

in vec4 v_position_viewspace;
in vec4 v_normal_viewspace;
in vec4 v_color;

out vec4 f_color;

void main() {
    // ambient
    float ambient_strength = 0.1;
    vec3 ambient = ambient_strength * u_light_color;
  	
    // diffuse 
    vec3 norm = normalize(v_normal_viewspace.xyz);
    vec3 light_dir = normalize(u_light_position_viewspace - v_position_viewspace.xyz);
    float diff = max(0.0, dot(norm, light_dir));
    vec3 diffuse = diff * u_light_color;
    
    // specular
    float specular_strength = 0.5;
    vec3 view_dir = vec3(0.0, 0.0, -1.0);
    vec3 reflect_dir = reflect(-light_dir, norm);
    float spec = pow(max(0.0, dot(view_dir, reflect_dir)), 32);
    vec3 specular = specular_strength * spec * u_light_color;

    f_color = v_color * vec4(ambient + diffuse + specular, 1.0);
}
";

static SKY_VS_SRC: &'static [u8] = b"
#version 450 core

uniform mat4 u_proj_matrix;
uniform mat4 u_modelview_matrix;

layout(location = 0) in vec4 a_position;

out vec3 v_tex_coords;

void main() {
    v_tex_coords = a_position.xyz;
    vec4 pos = u_proj_matrix * u_modelview_matrix * vec4(a_position.xyz, 1.0);
    gl_Position = pos.xyww; // Set z to 1
}
";

static SKY_FS_SRC: &'static [u8] = b"
#version 450 core

struct TextureSelector {
    uint tab;
    float layer;
};

uniform samplerCubeArray u_cubemaps[4]; // Solid 1x1, Low-res, Medium-res, Hi-res
uniform TextureSelector u_skybox;

in vec3 v_tex_coords;

out vec4 f_color;

void main() {
    f_color = texture(u_cubemaps[u_skybox.tab], vec4(v_tex_coords, u_skybox.layer));
}
";

fn new_gl_color_program() -> Result<gx::ProgramEx, String> {
    let vs = gx::VertexShader::try_from_source(VS_SRC)?;
    let fs = gx::FragmentShader::try_from_source(FS_SRC)?;
    let prog = gx::Program::try_from_vert_frag(&vs, &fs)?;
    Ok(gx::ProgramEx::new(prog))
}
fn new_gl_sky_program() -> Result<gx::ProgramEx, String> {
    let vs = gx::VertexShader::try_from_source(SKY_VS_SRC)?;
    let fs = gx::FragmentShader::try_from_source(SKY_FS_SRC)?;
    let prog = gx::Program::try_from_vert_frag(&vs, &fs)?;
    Ok(gx::ProgramEx::new(prog))
}
fn unwrap_or_display_error(r: Result<gx::ProgramEx, String>) -> gx::ProgramEx {
    match r {
        Ok(p) => p,
        Err(e) => {
            error!("GL compile error\n{}", e);
            panic!("GL compile error\n{}", e)
        },
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
pub struct GLSystem {
    viewport_size: Extent2<u32>,
    color_program: gx::ProgramEx,
    sky_program: gx::ProgramEx,
    mesh_vaos: HashMap<MeshID, gx::VertexArray>,
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
            color_program: unwrap_or_display_error(new_gl_color_program()),
            sky_program:   unwrap_or_display_error(new_gl_sky_program  ()),
            mesh_vaos: Default::default(),
            mesh_position_buffers: Default::default(),
            mesh_normal_buffers: Default::default(),
            mesh_color_buffers: Default::default(),
            mesh_index_buffers: Default::default(),
            pipeline_statistics_arb_queries,
        }
    }

    fn render_scene(&mut self, scene: &Scene, _draw: &Draw) {
        unsafe {
            gl::UseProgram(self.color_program.program().gl_id());
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

        self.color_program.set_uniform("u_proj_matrix", &[proj]);
        self.color_program.set_uniform("u_light_position_viewspace", &[Vec3::new(0., 0., 0.)]);
        self.color_program.set_uniform("u_light_color", &[Rgb::white()]);

        for &MeshInstance { ref mesh_id, xform } in scene.mesh_instances.values() {
            let mesh = &scene.meshes[mesh_id];
            let model = Mat4::from(xform);
            let modelview = view * model;
            let normal_matrix = modelview.inverted().transposed();
            self.color_program.set_uniform("u_modelview_matrix", &[modelview]);
            self.color_program.set_uniform("u_normal_matrix", &[normal_matrix]);

            /*
            unsafe {
                gl::Disable(gl::CULL_FACE);
                //gl::CullFace(gl::BACK);
            }
            */
            unsafe {
                gl::BindVertexArray(self.mesh_vaos[mesh_id].gl_id()); // FIXME: Filling them every time = not efficient
            }

            assert!(!mesh.vposition.is_empty());
            let pos_buffer = self.mesh_position_buffers.get(mesh_id).expect("Meshes must have a position buffer (for now)!");
            unsafe {
                gl::BindBuffer(gx::BufferTarget::Array as _, pos_buffer.gl_id());
                gl::EnableVertexAttribArray(VAttrib::PositionVec4f32 as _);
                gl::VertexAttribPointer(VAttrib::PositionVec4f32 as _, 4, gl::FLOAT, gl::FALSE, 4*4, 0 as _);
                gl::BindBuffer(gx::BufferTarget::Array as _, 0);
            }

            assert!(!mesh.vnormal.is_empty());
            let norm_buffer = self.mesh_normal_buffers.get(mesh_id).expect("Meshes must have a normals buffer (for now)!");
            unsafe {
                gl::BindBuffer(gx::BufferTarget::Array as _, norm_buffer.gl_id());
                gl::EnableVertexAttribArray(VAttrib::NormalVec4f32 as _);
                gl::VertexAttribPointer(VAttrib::NormalVec4f32 as _, 4, gl::FLOAT, gl::FALSE, 4*4, 0 as _);
                gl::BindBuffer(gx::BufferTarget::Array as _, 0);
            }

            let set_default_color = |rgba: Rgba<u8>| unsafe {
                let rgba = rgba.map(|x| x as f32) / 255.;
                gl::DisableVertexAttribArray(VAttrib::ColorRgbau8 as _);
                gl::VertexAttrib4f(VAttrib::ColorRgbau8 as _, rgba.r, rgba.g, rgba.b, rgba.a);
            };
            match self.mesh_color_buffers.get(mesh_id) {
                None => set_default_color(Rgba::white()),
                Some(col_buffer) => {
                    match mesh.vcolor.len() {
                        0 => set_default_color(Rgba::white()),
                        1 => set_default_color(mesh.vcolor[0]),
                        _ => unsafe {
                            gl::BindBuffer(gx::BufferTarget::Array as _, col_buffer.gl_id());
                            gl::EnableVertexAttribArray(VAttrib::ColorRgbau8 as _);
                            gl::VertexAttribPointer(VAttrib::ColorRgbau8 as _, 4, gl::FLOAT, gl::TRUE, 4, 0 as _);
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
            unsafe {
                gl::BindVertexArray(0);
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
                    self.mesh_vaos.entry(mesh_id).or_insert_with(gx::VertexArray::new);
                    gx_buffer_data_dsa(self.mesh_position_buffers.entry(mesh_id).or_insert_with(gx::Buffer::new), vposition, gx::BufferUsage::StaticDraw);
                    gx_buffer_data_dsa(self.mesh_normal_buffers.entry(mesh_id).or_insert_with(gx::Buffer::new), vnormal, gx::BufferUsage::StaticDraw);
                    if vcolor.is_empty() {
                        self.mesh_color_buffers.remove(&mesh_id);
                    } else {
                        gx_buffer_data_dsa(self.mesh_color_buffers.entry(mesh_id).or_insert_with(gx::Buffer::new), vcolor, gx::BufferUsage::StaticDraw);
                    }
                    if indices.is_empty() {
                        self.mesh_index_buffers.remove(&mesh_id);
                    } else {
                        gx_buffer_data_dsa(self.mesh_index_buffers.entry(mesh_id).or_insert_with(gx::Buffer::new), indices, gx::BufferUsage::StaticDraw);
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

