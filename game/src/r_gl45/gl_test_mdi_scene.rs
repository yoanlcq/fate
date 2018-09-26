use std::ptr;
use std::mem;
use std::ops::Range;
use fate::math::{Vec2, Vec3, Vec4, Mat4, Rgba, Rgb};
use fate::gx::{self, Object, {gl::{self, types::*}}};
use mesh::VertexAttribIndex;
use camera::View;

macro_rules! hashmap {
    ($($key:expr => $value:expr),*) => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )*
            m
        }
    };
}

const MAX_VERTICES : isize = 1024 << 4;
const MAX_INSTANCES: isize = 4096;
const MAX_INDICES  : isize = 1024 << 5;
const MAX_CMDS     : isize = 1024;
const MAX_MATERIALS: isize = 16384 / mem::size_of::<Material>() as isize; // min value in bytes of GL_MAX_UNIFORM_BLOCK_SIZE (limit does not apply to SSBOs)
const MAX_POINT_LIGHTS: isize = 32;

#[derive(Debug)]
pub struct GLTestMDIScene {
    vao: gx::VertexArray,
    position_vbo: gx::Buffer,
    normal_vbo: gx::Buffer,
    uv_vbo: gx::Buffer,
    weight_vbo: gx::Buffer,
    joint_vbo: gx::Buffer,
    model_matrix_vbo: gx::Buffer,
    material_index_vbo: gx::Buffer,
    ibo: gx::Buffer,
    cmd_buffer: gx::Buffer,
    material_buffer: gx::Buffer,
    point_light_buffer: gx::Buffer,
    program: gx::ProgramEx,
    heap_info: HeapInfo,
}

impl GLTestMDIScene {
    pub fn new() -> Self {
        unsafe {
            Self::new_unsafe()
        }
    }
    unsafe fn new_unsafe() -> Self {
        let vao = gx::VertexArray::new();
        let mut buffers = [0; 11];
        gl::CreateBuffers(buffers.len() as _, buffers.as_mut_ptr());
        let position_vbo = buffers[0];
        let normal_vbo = buffers[1];
        let uv_vbo = buffers[2];
        let weight_vbo = buffers[3];
        let joint_vbo = buffers[4];
        let model_matrix_vbo = buffers[5];
        let material_index_vbo = buffers[6];
        let ibo = buffers[7];
        let cmd_buffer = buffers[8];
        let material_buffer = buffers[9];
        let point_light_buffer = buffers[10];

        let flags = gl::DYNAMIC_STORAGE_BIT;
        gl::NamedBufferStorage(position_vbo, MAX_VERTICES * 3 * 4, ptr::null(), flags);
        gl::NamedBufferStorage(normal_vbo, MAX_VERTICES * 3 * 4, ptr::null(), flags);
        gl::NamedBufferStorage(uv_vbo, MAX_VERTICES * 2 * 4, ptr::null(), flags);
        gl::NamedBufferStorage(weight_vbo, MAX_VERTICES * 4 * 4, ptr::null(), flags);
        gl::NamedBufferStorage(joint_vbo, MAX_VERTICES * 4 * 2, ptr::null(), flags);
        gl::NamedBufferStorage(model_matrix_vbo, MAX_INSTANCES * 4 * 4 * 4, ptr::null(), flags);
        gl::NamedBufferStorage(material_index_vbo, MAX_INSTANCES * 2, ptr::null(), flags);
        gl::NamedBufferStorage(ibo, MAX_INDICES * 4, ptr::null(), flags);
        gl::NamedBufferStorage(cmd_buffer, MAX_CMDS * mem::size_of::<GLDrawElementsIndirectCommand>() as isize, ptr::null(), flags);
        gl::NamedBufferStorage(material_buffer, MAX_MATERIALS * mem::size_of::<Material>() as isize, ptr::null(), flags);
        gl::NamedBufferStorage(point_light_buffer, MAX_POINT_LIGHTS * mem::size_of::<PointLight>() as isize, ptr::null(), flags);

        // Specifying vertex attrib layout

        gl::BindVertexArray(vao.gl_id());
        gl::EnableVertexAttribArray(VertexAttribIndex::Position as _);
        gl::EnableVertexAttribArray(VertexAttribIndex::Normal as _);
        gl::EnableVertexAttribArray(VertexAttribIndex::UV as _);
        gl::EnableVertexAttribArray(VertexAttribIndex::Weights as _);
        gl::EnableVertexAttribArray(VertexAttribIndex::Joints as _);
        gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 0);
        gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 1);
        gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 2);
        gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 3);
        gl::EnableVertexAttribArray(VertexAttribIndex::MaterialIndex as _);

        gl::VertexAttribDivisor(VertexAttribIndex::Position as _, 0);
        gl::VertexAttribDivisor(VertexAttribIndex::Normal as _, 0);
        gl::VertexAttribDivisor(VertexAttribIndex::UV as _, 0);
        gl::VertexAttribDivisor(VertexAttribIndex::Weights as _, 0);
        gl::VertexAttribDivisor(VertexAttribIndex::Joints as _, 0);
        gl::VertexAttribDivisor(VertexAttribIndex::ModelMatrix as GLuint + 0, 1);
        gl::VertexAttribDivisor(VertexAttribIndex::ModelMatrix as GLuint + 1, 1);
        gl::VertexAttribDivisor(VertexAttribIndex::ModelMatrix as GLuint + 2, 1);
        gl::VertexAttribDivisor(VertexAttribIndex::ModelMatrix as GLuint + 3, 1);
        gl::VertexAttribDivisor(VertexAttribIndex::MaterialIndex as _, 1);

        gl::BindBuffer(gl::ARRAY_BUFFER, position_vbo);
        gl::VertexAttribPointer(VertexAttribIndex::Position as _, 3, gl::FLOAT, gl::FALSE, 0, 0 as _);
        gl::BindBuffer(gl::ARRAY_BUFFER, normal_vbo);
        gl::VertexAttribPointer(VertexAttribIndex::Normal as _, 3, gl::FLOAT, gl::FALSE, 0, 0 as _);
        gl::BindBuffer(gl::ARRAY_BUFFER, uv_vbo);
        gl::VertexAttribPointer(VertexAttribIndex::UV as _, 2, gl::FLOAT, gl::FALSE, 0, 0 as _);
        gl::BindBuffer(gl::ARRAY_BUFFER, weight_vbo);
        gl::VertexAttribPointer(VertexAttribIndex::Weights as _, 4, gl::FLOAT, gl::FALSE, 0, 0 as _);
        gl::BindBuffer(gl::ARRAY_BUFFER, joint_vbo);
        gl::VertexAttribPointer(VertexAttribIndex::Joints as _, 4, gl::UNSIGNED_SHORT, gl::FALSE, 0, 0 as _);
        gl::BindBuffer(gl::ARRAY_BUFFER, model_matrix_vbo);
        gl::VertexAttribPointer(VertexAttribIndex::ModelMatrix as GLuint + 0, 4, gl::FLOAT, gl::FALSE, 4*4*4, (0*4*4) as _);
        gl::VertexAttribPointer(VertexAttribIndex::ModelMatrix as GLuint + 1, 4, gl::FLOAT, gl::FALSE, 4*4*4, (1*4*4) as _);
        gl::VertexAttribPointer(VertexAttribIndex::ModelMatrix as GLuint + 2, 4, gl::FLOAT, gl::FALSE, 4*4*4, (2*4*4) as _);
        gl::VertexAttribPointer(VertexAttribIndex::ModelMatrix as GLuint + 3, 4, gl::FLOAT, gl::FALSE, 4*4*4, (3*4*4) as _);
        gl::BindBuffer(gl::ARRAY_BUFFER, material_index_vbo);
        gl::VertexAttribIPointer(VertexAttribIndex::MaterialIndex as _, 1, gl::UNSIGNED_SHORT, 0, 0 as _);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        let mut s = Self {
            vao,
            position_vbo: gx::Buffer::from_gl_id(position_vbo),
            normal_vbo: gx::Buffer::from_gl_id(normal_vbo),
            uv_vbo: gx::Buffer::from_gl_id(uv_vbo),
            weight_vbo: gx::Buffer::from_gl_id(weight_vbo),
            joint_vbo: gx::Buffer::from_gl_id(joint_vbo),
            model_matrix_vbo: gx::Buffer::from_gl_id(model_matrix_vbo),
            material_index_vbo: gx::Buffer::from_gl_id(material_index_vbo),
            ibo: gx::Buffer::from_gl_id(ibo),
            cmd_buffer: gx::Buffer::from_gl_id(cmd_buffer),
            material_buffer: gx::Buffer::from_gl_id(material_buffer),
            point_light_buffer: gx::Buffer::from_gl_id(point_light_buffer),
            program: super::new_program_ex_unwrap(PBR_VS, PBR_FS),
            heap_info: HeapInfo::default(),
        };
        s.add_meshes();
        s
    }
    unsafe fn add_meshes(&mut self) {
        use ::std::collections::HashMap;

        let (mesh1_positions, mesh1_positions_morph1) = {
            let positions_orig = [
                Vec3::<f32>::new(0., 0., 0.),
                Vec3::<f32>::new(1., 0., 0.),
                Vec3::<f32>::new(0., 1., 0.),
            ];

            // Per-mesh
            let morphtarget_displacements = [
                hashmap!(2 => Vec3::<f32>::new(0.7, 0., 0.)),
                hashmap!(0 => Vec3::<f32>::new(0., -1., 0.)),
            ];

            // Per morphed instance
            let morphtarget_weights = [ 1., 0.5 ];

            let mut positions = positions_orig.clone();
            for (i, pos) in positions.iter_mut().enumerate() {
                let mut displacement = Vec3::zero();
                for (target_i, weight) in morphtarget_weights.iter().enumerate() {
                    displacement += morphtarget_displacements[target_i].get(&i).map(|v| *v).unwrap_or_default() * *weight;
                }
                *pos += displacement;
            }
            (positions_orig, positions)
        };

        let positions = [
            // Mesh 1
            mesh1_positions[0],
            mesh1_positions[1],
            mesh1_positions[2],

            // Mesh 2
            Vec3::<f32>::new( 0.0, 1.0, 0.),
            Vec3::<f32>::new(-0.5, 0.0, 0.),
            Vec3::<f32>::new( 0.5, 0.0, 0.),

            // Mesh 1, morphed 1
            mesh1_positions_morph1[0],
            mesh1_positions_morph1[1],
            mesh1_positions_morph1[2],
        ];
        let normals = [
            Vec3::<f32>::new(-1., -1., -1.),
            Vec3::<f32>::new(1., 0., -1.),
            Vec3::<f32>::new(0., 1., -1.),

            Vec3::<f32>::new( 0., 1., -1.),
            Vec3::<f32>::new(-1., 0., -1.),
            Vec3::<f32>::new( 1., 0., -1.),

            Vec3::<f32>::new(-1., -1., -1.),
            Vec3::<f32>::new(1., 0., -1.),
            Vec3::<f32>::new(0., 1., -1.),
        ];
        let uvs = [
            Vec2::<f32>::new(0., 0.),
            Vec2::<f32>::new(1., 0.),
            Vec2::<f32>::new(0., 1.),

            Vec2::<f32>::new(0.5, 1.),
            Vec2::<f32>::new(0., 0.),
            Vec2::<f32>::new(1., 0.),

            Vec2::<f32>::new(0., 0.),
            Vec2::<f32>::new(1., 0.),
            Vec2::<f32>::new(0., 1.),
        ];
        let weights = [
            Vec4::<f32>::new(1., 0., 0., 0.),
            Vec4::<f32>::new(1., 0., 0., 0.),
            Vec4::<f32>::new(1., 0., 0., 0.),
            Vec4::<f32>::new(1., 0., 0., 0.),
            Vec4::<f32>::new(1., 0., 0., 0.),
            Vec4::<f32>::new(1., 0., 0., 0.),
            Vec4::<f32>::new(1., 0., 0., 0.),
            Vec4::<f32>::new(1., 0., 0., 0.),
            Vec4::<f32>::new(1., 0., 0., 0.),
        ];
        let joints = [
            Vec4::<u16>::new(0, 0, 0, 0),
            Vec4::<u16>::new(0, 0, 0, 0),
            Vec4::<u16>::new(0, 0, 0, 0),

            Vec4::<u16>::new(0, 0, 0, 0),
            Vec4::<u16>::new(0, 0, 0, 0),
            Vec4::<u16>::new(0, 0, 0, 0),

            Vec4::<u16>::new(0, 0, 0, 0),
            Vec4::<u16>::new(0, 0, 0, 0),
            Vec4::<u16>::new(0, 0, 0, 0),
        ];
        let indices = [
            0_u32, 1, 2,
            0_u32, 1, 2,
            0_u32, 1, 2,
        ];

        let model_matrices = [
            Mat4::<f32>::translation_3d(Vec3::new(-1.0, 0., 0.)),
            Mat4::<f32>::translation_3d(Vec3::new( 0.0, 0., 0.)),
            Mat4::<f32>::translation_3d(Vec3::new( 1.0, 0., 0.)),

            Mat4::<f32>::translation_3d(Vec3::new(-1.0, 1., 0.)),
            Mat4::<f32>::translation_3d(Vec3::new( 0.0, 1., 0.)),
            Mat4::<f32>::translation_3d(Vec3::new( 1.0, 1., 0.)),
        ];
        let material_indices = [
            0_u16, 1, 2,
            3, 4, 5,
        ];

        // Check weights
        for weights in weights.iter().cloned() {
            assert_relative_eq!(weights.sum(), 1.);
        }

        gl::NamedBufferSubData(self.position_vbo.gl_id(), 0, mem::size_of_val(&positions[..]) as _, positions.as_ptr() as _);
        gl::NamedBufferSubData(self.normal_vbo.gl_id(), 0, mem::size_of_val(&normals[..]) as _, normals.as_ptr() as _);
        gl::NamedBufferSubData(self.uv_vbo.gl_id(), 0, mem::size_of_val(&uvs[..]) as _, uvs.as_ptr() as _);
        gl::NamedBufferSubData(self.weight_vbo.gl_id(), 0, mem::size_of_val(&weights[..]) as _, weights.as_ptr() as _);
        gl::NamedBufferSubData(self.joint_vbo.gl_id(), 0, mem::size_of_val(&joints[..]) as _, joints.as_ptr() as _);
        gl::NamedBufferSubData(self.model_matrix_vbo.gl_id(), 0, mem::size_of_val(&model_matrices[..]) as _, model_matrices.as_ptr() as _);
        gl::NamedBufferSubData(self.material_index_vbo.gl_id(), 0, mem::size_of_val(&material_indices[..]) as _, material_indices.as_ptr() as _);
        gl::NamedBufferSubData(self.ibo.gl_id(), 0, mem::size_of_val(&indices[..]) as _, indices.as_ptr() as _);

        self.heap_info = HeapInfo {
            vertex_ranges: vec![0..3, 3..6, 6..9],
            index_ranges: vec![0..3, 3..6, 6..9],
            instance_ranges: vec![0..3, 3..6, 0..2],
            instance_range_mesh_entry: vec![0, 1, 2],
        };
    }
    pub fn draw(&self, view: &View, texture2d_arrays: &[GLuint]) {
        unsafe {
            self.draw_unsafe(view, texture2d_arrays)
        }
    }
    unsafe fn draw_unsafe(&self, view: &View, texture2d_arrays: &[GLuint]) {

        let joint_matrices = [Mat4::<f32>::identity(); 32]; // FIXME: But this changes on a per-instance basis (driven by animation)

        assert!(texture2d_arrays.len() <= 16, "Too many texture2d arrays for shader");

        // FIXME: Hardcoded texture selectors
        let materials = [
            Material { albedo_mul: Rgba::red()   , albedo_map: (2 << 16) | 0, metallic_mul: 1., metallic_map: 1, roughness_mul: 0.4, roughness_map: 1, ao_map: 1, normal_map: 0, _pad: Default::default(), },
            Material { albedo_mul: Rgba::yellow(), albedo_map: (2 << 16) | 0, metallic_mul: 1., metallic_map: 1, roughness_mul: 0.4, roughness_map: 1, ao_map: 1, normal_map: 0, _pad: Default::default(), },
            Material { albedo_mul: Rgba::green() , albedo_map: (2 << 16) | 1, metallic_mul: 1., metallic_map: 1, roughness_mul: 0.4, roughness_map: 1, ao_map: 1, normal_map: 0, _pad: Default::default(), },
            Material { albedo_mul: Rgba::white() , albedo_map: (2 << 16) | 1, metallic_mul: 1., metallic_map: 1, roughness_mul: 0.4, roughness_map: 1, ao_map: 1, normal_map: 0, _pad: Default::default(), },
            Material { albedo_mul: Rgba::white() , albedo_map: (2 << 16) | 2, metallic_mul: 1., metallic_map: 1, roughness_mul: 0.4, roughness_map: 1, ao_map: 1, normal_map: 0, _pad: Default::default(), },
            Material { albedo_mul: Rgba::cyan()  , albedo_map: (2 << 16) | 2, metallic_mul: 1., metallic_map: 1, roughness_mul: 0.4, roughness_map: 1, ao_map: 1, normal_map: 0, _pad: Default::default(), },
        ];

        gl::NamedBufferSubData(self.material_buffer.gl_id(), 0, mem::size_of_val(&materials[..]) as _, materials.as_ptr() as _);
        let nb_materials = materials.len();

        let point_lights = [
            PointLight {
                position: Vec4::new(-5., 0., -5., 1.),
                color: Rgba::white(),
                range: 5.,
                attenuation_factor: 0.9,
                pad: Default::default(),
            },
        ];
        gl::NamedBufferSubData(self.point_light_buffer.gl_id(), 0, mem::size_of_val(&point_lights[..]) as _, point_lights.as_ptr() as _);
        let nb_point_lights = point_lights.len();

        let mut cmds = vec![];

        let m = &self.heap_info;
        for (i, mesh) in m.instance_ranges.iter().zip(m.instance_range_mesh_entry.iter()) {
            let index_range = &m.index_ranges[*mesh as usize];
            let vertex_range = &m.vertex_ranges[*mesh as usize];
            cmds.push(GLDrawElementsIndirectCommand {
                base_instance: i.start,
                nb_instances: i.end - i.start,
                first_index: index_range.start, // Offset into the index buffer
                nb_indices: index_range.end - index_range.start,
                base_vertex: vertex_range.start, // Value added to indices for vertex retrieval
            });
        }
        let nb_cmds = cmds.len();
        gl::NamedBufferSubData(self.cmd_buffer.gl_id(), 0, mem::size_of_val(&cmds[..]) as _, cmds.as_ptr() as _); // PERF

        gl::BindTextures(0, texture2d_arrays.len() as _, texture2d_arrays.as_ptr());
        let units = [0_i32, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, ];

        gl::UseProgram(self.program.inner().gl_id());
        self.program.set_uniform_primitive("u_joint_matrices[0]", &joint_matrices[..]);
        self.program.set_uniform("u_texture2d_arrays[0]", gx::GLSLType::Sampler2DArray, &units[..texture2d_arrays.len()]);
        self.program.set_uniform_primitive("u_viewproj_matrix", &[view.proj_matrix() * view.view_matrix()]);
        self.program.set_uniform_primitive("u_eye_position_worldspace", &[view.xform.position]);
        self.program.set_uniform_primitive("u_directional_light.direction", &[Vec3::<f32>::new(1., 1., 1.).normalized()]);
        self.program.set_uniform_primitive("u_directional_light.color", &[Rgb::<f32>::black()]);
        gl::BindBufferRange(gl::SHADER_STORAGE_BUFFER, 1, self.point_light_buffer.gl_id(), 0, (nb_point_lights * mem::size_of::<PointLight>()) as _);
        gl::BindBufferRange(gl::SHADER_STORAGE_BUFFER, 2, self.material_buffer.gl_id(), 0, (nb_materials * mem::size_of::<Material>()) as _);

        gl::BindVertexArray(self.vao.gl_id());
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo.gl_id());
        gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, self.cmd_buffer.gl_id()); // In core profile, we MUST use a buffer to store commands
        gl::MultiDrawElementsIndirect(gl::TRIANGLES, gl::UNSIGNED_INT, 0 as _, nb_cmds as _, 0);
        gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, 0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, 0);
        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, 0);
        gl::UseProgram(0);

        gl::BindTextures(0, texture2d_arrays.len() as _, ptr::null());
    }
}

#[derive(Debug, Default)]
pub struct HeapInfo {
    // Indexed by mesh
    pub vertex_ranges: Vec<Range<u32>>,
    pub index_ranges: Vec<Range<u32>>,

    // Indexed by instance
    pub instance_ranges: Vec<Range<u32>>,
    pub instance_range_mesh_entry: Vec<u32>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Material {
    pub albedo_mul   : Rgba<f32>,
    pub albedo_map   : u32,
    pub normal_map   : u32,
    pub metallic_mul : f32,
    pub metallic_map : u32,
    pub roughness_mul: f32,
    pub roughness_map: u32,
    pub ao_map       : u32,
    pub _pad: u32,
}

assert_eq_size!(material_struct_size; Material, [Vec4<f32>; 3]);

#[derive(Debug, Default, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct DirectionalLight {
    pub direction: Vec3<f32>,
    pub color: Rgb<f32>,
}


#[derive(Debug, Default, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct PointLight {
    pub position: Vec4<f32>,
    pub color: Rgba<f32>,
    pub range: f32,
    pub attenuation_factor: f32,
    pub pad: Vec2<f32>,
}

assert_eq_size!(point_light_struct_size; PointLight, [Vec4<f32>; 3]);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(C)]
pub struct GLDrawElementsIndirectCommand {
    pub nb_indices: GLuint,
    pub nb_instances: GLuint,
    pub first_index: GLuint,
    pub base_vertex: GLuint,
    pub base_instance: GLuint,
}

static PBR_VS : &'static [u8] = 
b"#version 450 core

uniform mat4 u_viewproj_matrix;
uniform mat4 u_joint_matrices[32];

layout(location =  0) in vec3 a_position;
layout(location =  1) in vec3 a_normal;
layout(location =  5) in vec2 a_uv;
layout(location =  9) in vec4 a_weights;
layout(location = 10) in vec4 a_joints;
layout(location = 11) in mat4 a_model_matrix;
layout(location = 15) in uint a_material_index;

out vec3 v_position_worldspace;
out vec3 v_normal;
out vec2 v_uv;
flat out uint v_material_index;

void main() {
    mat4 skin_matrix =
        a_weights.x * u_joint_matrices[int(a_joints.x)] +
        a_weights.y * u_joint_matrices[int(a_joints.y)] +
        a_weights.z * u_joint_matrices[int(a_joints.z)] +
        a_weights.w * u_joint_matrices[int(a_joints.w)];

    mat4 final_model_matrix = a_model_matrix * skin_matrix;

    vec4 world_pos = final_model_matrix * vec4(a_position, 1.0);

    gl_Position = u_viewproj_matrix * vec4(world_pos.xyz, 1.0);
    v_position_worldspace = world_pos.xyz;
    v_normal = mat3(transpose(inverse(final_model_matrix))) * a_normal; // FIXME PERF
    v_uv = a_uv;
    v_material_index = a_material_index;
}
";

static PBR_FS: &'static [u8] = 
b"#version 450 core

const float PI = 3.14159265359;

struct Material {
    vec4  albedo_mul;
    uint  albedo_map;
    uint  normal_map;
    float metallic_mul;
    uint  metallic_map;
    float roughness_mul;
    uint  roughness_map;
    uint  ao_map;
};

struct PointLight {
    vec4 position;
    vec4 color;
    float range;
    float attenuation_factor;
    vec2 _pad;
};

struct DirectionalLight {
    vec3 direction;
    vec3 color;
};


uniform sampler2DArray u_texture2d_arrays[16];
uniform vec3 u_eye_position_worldspace;
uniform DirectionalLight u_directional_light;
layout(std430, binding = 1) buffer PointLights { PointLight u_point_lights[]; };
layout(std430, binding = 2) buffer Materials { Material u_materials[]; };

in vec3 v_position_worldspace;
in vec3 v_normal;
in vec2 v_uv;
flat in uint v_material_index;

out vec4 f_color;

vec4 tex(uint sel, vec2 uv) {
    return texture(u_texture2d_arrays[sel >> 16], vec3(uv, float(sel & 0xffffu)));
}

// Computes ratio between specular and diffuse reflection
// F0 is the surface reflection at zero incidence (i.e when looking directly at the surface)
vec3 fresnel_schlick(float cos_theta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(1.0 - cos_theta, 5.0);
}

// Computes the normal distribution coefficient, D
float distribution_ggx(vec3 N, vec3 H, float roughness) {
    float a      = roughness*roughness;
    float a2     = a*a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;
	
    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
	
    return num / denom;
}

float geometry_schlick_ggx(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float num   = NdotV;
    float denom = NdotV * (1.0 - k) + k;
	
    return num / denom;
}

// Computes the geometry coefficient, G
float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2  = geometry_schlick_ggx(NdotV, roughness);
    float ggx1  = geometry_schlick_ggx(NdotL, roughness);
	
    return ggx1 * ggx2;
}

void main() {
    vec3 N = normalize(v_normal);
    vec3 V = normalize(u_eye_position_worldspace - v_position_worldspace);

#define mat u_materials[v_material_index]

    vec4 albedo = mat.albedo_mul * tex(mat.albedo_map, v_uv);
    float metallic = mat.metallic_mul * tex(mat.metallic_map, v_uv).r;
    float roughness = mat.roughness_mul * tex(mat.roughness_map, v_uv).r;
    float ao = tex(mat.ao_map, v_uv).r;

    vec3 F0 = vec3(0.04); // See fresnel_schlick
    F0 = mix(F0, albedo.rgb, metallic);

    vec3 Lo = vec3(0.0);
    for (int i = 0; i < u_point_lights.length(); ++i) {
        vec3 L_unnormalized = u_point_lights[i].position.xyz - v_position_worldspace;
        float distance = length(L_unnormalized);

        vec3 L = L_unnormalized / distance;
        vec3 H = normalize(V + L);

        float attenuation = 1.0 / (distance * distance); // XXX attenuation because we're in linear space, which we gamma correct at end of the shader
        vec3 radiance = u_point_lights[i].color.rgb * attenuation;

        float NDF = distribution_ggx(N, H, roughness);
        float G = geometry_smith(N, V, L, roughness);
        vec3 F = fresnel_schlick(max(0.0, dot(H, V)), F0);

        // Cook-Torrance BRDF
        vec3 numerator = NDF * G * F;
        float denominator = 4.0 * max(0.0, dot(N, V)) * max(0.0, dot(N, L));
        vec3 specular = numerator / max(denominator, 0.001);

        //
        vec3 Ks = F;
        vec3 Kd = (vec3(1.0) - Ks) * (1.0 - metallic);

        //
        float NdotL = max(0.0, dot(N, L));
        Lo += (Kd * albedo.rgb / PI + specular) * radiance * NdotL;
    }

    vec3 ambient = vec3(0.03) * albedo.rgb * ao;
    vec3 color = ambient + Lo;

    // Pretend directional light is used
    color += u_directional_light.color * 0.0001;

    // Gamma correct
    color /= color + vec3(1.0);
    color = pow(color, vec3(1.0/2.2));

    f_color = vec4(color, albedo.a);
}
";
