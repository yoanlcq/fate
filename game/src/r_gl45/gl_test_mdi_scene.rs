use std::ptr;
use std::mem;
use std::ops::Range;
use fate::math::{Vec2, Vec3, Vec4, Mat4, Rgba, Rgb};
use fate::gx::{self, Object, {gl::{self, types::*}}};
use mesh::VertexAttribIndex;
use camera::View;

// TODO:
// - Material textures
// - Shape keys
// - Skeletal animations
// - Full PBR rendering
// | Lights

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
        let mut buffers = [0; 9];
        gl::CreateBuffers(buffers.len() as _, buffers.as_mut_ptr());
        let position_vbo = buffers[0];
        let normal_vbo = buffers[1];
        let uv_vbo = buffers[2];
        let model_matrix_vbo = buffers[3];
        let material_index_vbo = buffers[4];
        let ibo = buffers[5];
        let cmd_buffer = buffers[6];
        let material_buffer = buffers[7];
        let point_light_buffer = buffers[8];

        let flags = gl::DYNAMIC_STORAGE_BIT;
        gl::NamedBufferStorage(position_vbo, MAX_VERTICES * 3 * 4, ptr::null(), flags);
        gl::NamedBufferStorage(normal_vbo, MAX_VERTICES * 3 * 4, ptr::null(), flags);
        gl::NamedBufferStorage(uv_vbo, MAX_VERTICES * 2 * 4, ptr::null(), flags);
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
        gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 0);
        gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 1);
        gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 2);
        gl::EnableVertexAttribArray(VertexAttribIndex::ModelMatrix as GLuint + 3);
        gl::EnableVertexAttribArray(VertexAttribIndex::MaterialIndex as _);

        gl::VertexAttribDivisor(VertexAttribIndex::Position as _, 0);
        gl::VertexAttribDivisor(VertexAttribIndex::Normal as _, 0);
        gl::VertexAttribDivisor(VertexAttribIndex::UV as _, 0);
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
        let positions = [
            Vec3::<f32>::new(0., 0., 0.),
            Vec3::<f32>::new(1., 0., 0.),
            Vec3::<f32>::new(0., 1., 0.),

            Vec3::<f32>::new( 0.0, 1.0, 0.),
            Vec3::<f32>::new(-0.5, 0.0, 0.),
            Vec3::<f32>::new( 0.5, 0.0, 0.),
        ];
        let normals = [
            Vec3::<f32>::new(-1., -1., -1.),
            Vec3::<f32>::new(1., 0., -1.),
            Vec3::<f32>::new(0., 1., -1.),

            Vec3::<f32>::new( 0., 1., -1.),
            Vec3::<f32>::new(-1., 0., -1.),
            Vec3::<f32>::new( 1., 0., -1.),
        ];
        let uvs = [
            Vec2::<f32>::new(0., 0.),
            Vec2::<f32>::new(1., 0.),
            Vec2::<f32>::new(0., 1.),

            Vec2::<f32>::new(0., 0.),
            Vec2::<f32>::new(1., 0.),
            Vec2::<f32>::new(0., 1.),
        ];
        let indices = [
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

        gl::NamedBufferSubData(self.position_vbo.gl_id(), 0, mem::size_of_val(&positions[..]) as _, positions.as_ptr() as _);
        gl::NamedBufferSubData(self.normal_vbo.gl_id(), 0, mem::size_of_val(&normals[..]) as _, normals.as_ptr() as _);
        gl::NamedBufferSubData(self.uv_vbo.gl_id(), 0, mem::size_of_val(&uvs[..]) as _, uvs.as_ptr() as _);
        gl::NamedBufferSubData(self.model_matrix_vbo.gl_id(), 0, mem::size_of_val(&model_matrices[..]) as _, model_matrices.as_ptr() as _);
        gl::NamedBufferSubData(self.material_index_vbo.gl_id(), 0, mem::size_of_val(&material_indices[..]) as _, material_indices.as_ptr() as _);
        gl::NamedBufferSubData(self.ibo.gl_id(), 0, mem::size_of_val(&indices[..]) as _, indices.as_ptr() as _);

        self.heap_info.vertex_ranges.push(0 .. 3);
        self.heap_info.index_ranges.push(0 .. 3);
        self.heap_info.vertex_ranges.push(3 .. 6);
        self.heap_info.index_ranges.push(3 .. 6);
        self.heap_info.instance_ranges.push(0 .. 3);
        self.heap_info.instance_range_mesh_entry.push(0);
        self.heap_info.instance_ranges.push(3 .. 6);
        self.heap_info.instance_range_mesh_entry.push(1);
    }
    pub fn draw(&self, view: &View, texture2d_arrays: &[GLuint]) {
        unsafe {
            self.draw_unsafe(view, texture2d_arrays)
        }
    }
    unsafe fn draw_unsafe(&self, view: &View, texture2d_arrays: &[GLuint]) {
        assert!(texture2d_arrays.len() <= 16, "Too many texture2d arrays for shader");

        // FIXME: Hardcoded texture selectors
        let materials = [
            Material { albedo_mul: Rgba::red()   , albedo_map: (2 << 16) | 0, .. Default::default() },
            Material { albedo_mul: Rgba::yellow(), albedo_map: (2 << 16) | 0, .. Default::default() },
            Material { albedo_mul: Rgba::green() , albedo_map: (2 << 16) | 1, .. Default::default() },
            Material { albedo_mul: Rgba::white() , albedo_map: (2 << 16) | 1, .. Default::default() },
            Material { albedo_mul: Rgba::white() , albedo_map: (2 << 16) | 2, .. Default::default() },
            Material { albedo_mul: Rgba::cyan()  , albedo_map: (2 << 16) | 2, .. Default::default() },
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

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_normal;
layout(location = 5) in vec2 a_uv;
layout(location = 9) in mat4 a_model_matrix;
layout(location = 13) in uint a_material_index;

out vec3 v_position_worldspace;
out vec3 v_normal;
out vec2 v_uv;
flat out uint v_material_index;

void main() {
    vec4 world_pos = a_model_matrix * vec4(a_position, 1.0);
    gl_Position = u_viewproj_matrix * world_pos;
    v_position_worldspace = world_pos.xyz;
    v_normal = mat3(transpose(inverse(a_model_matrix))) * a_normal; // FIXME PERF
    v_uv = a_uv;
    v_material_index = a_material_index;
}
";

static PBR_FS: &'static [u8] = 
b"#version 450 core

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

void main() {
    vec3 N = normalize(v_normal);
    vec3 V = normalize(u_eye_position_worldspace - v_position_worldspace);

#define mat u_materials[v_material_index]

    vec3 Kd = mat.albedo_mul.rgb * tex(mat.albedo_map, v_uv).rgb;
    vec3 Ks = vec3(1.0);

    vec3 color = vec3(0.0);

    vec3 L  = u_directional_light.direction; // Expected to be normalized
    vec3 Li = u_directional_light.color;
    vec3 H  = normalize(V + L);
    color += Li*(Kd*dot(L, N) + max(vec3(0), Ks*dot(H, N)));

    for(uint i = 0u; i < u_point_lights.length(); ++i) {
        vec3 L_unnormalized = u_point_lights[i].position.xyz - v_position_worldspace;
        float distance = length(L_unnormalized);

        L = L_unnormalized / distance;
        H = normalize(V + L);
        Li = u_point_lights[i].color.rgb / (u_point_lights[i].attenuation_factor * pow(max(1, distance / u_point_lights[i].range), 2));
        color += Li*(Kd*dot(L, N) + max(vec3(0), Ks*dot(H, N)));
    }

    f_color = vec4(color, 1.0); // TODO: alpha
}
";

// Lol no PBR
/*
// https://learnopengl.com/PBR/Lighting
static PBR_FS: &'static [u8] = 
"#version 450 core

in vec3 v_position;
in vec3 v_normal;
in vec2 v_uv;
flat in uint v_material_index;

out vec4 f_color;

struct Light {

};

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

layout(std430, binding = 1) buffer Lights { Light u_lights[]; };
layout(std430, binding = 2) buffer Materials { Material u_materials[]; };
uniform sampler2DArray u_texture2d_arrays[32];
uniform vec3 u_eye_position;

const float PI = 3.14159265359;

vec3 fresnel_schlick(float cos_theta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(1.0 - cos_theta, 5.0);
}  

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

float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2  = GeometrySchlickGGX(NdotV, roughness);
    float ggx1  = GeometrySchlickGGX(NdotL, roughness);
	
    return ggx1 * ggx2;
}

vec3 map_normal(vec3 N, vec3 sampled) {

}

vec4 tex(uint tex, vec2 uv) {
    return texture(u_texture2d_arrays[tex & 0xffff], vec3(uv, float(tex >> 16)));
}

void main() {

    vec3 N = normalize(v_normal);
    vec3 V = normalize(u_eye_position - v_position);

#define m u_materials[a_material_index]
    vec3  albedo    = m.albedo_mul * pow(tex(m.albedo_map, v_uv).rgb, 2.2); // Map sRGB to linear
    vec3  normal    = map_normal(N, tex(m.normal_map, v_uv).rgb);
    float metallic  = m.metallic_mul * tex(m.metallic_map, v_uv).r;
    float roughness = m.roughness_mul * tex(m.roughness_map, v_uv).r;
    float ao        = tex(m.ao_map, v_uv).r;
#undef m

    vec3 F0 = vec3(0.04); 
    F0 = mix(F0, albedo, metallic);
	           
    // reflectance equation
    vec3 Lo = vec3(0.0);
    for(int i = 0; i < u_lights.length(); ++i) 
    {
        // calculate per-light radiance
        vec3 L = normalize(lightPositions[i] - WorldPos);
        vec3 H = normalize(V + L);
        float distance    = length(lightPositions[i] - WorldPos);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance     = lightColors[i] * attenuation;        
        
        // cook-torrance brdf
        float NDF = DistributionGGX(N, H, roughness);        
        float G   = GeometrySmith(N, V, L, roughness);      
        vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);       
        
        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic;	  
        
        vec3 numerator    = NDF * G * F;
        float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0);
        vec3 specular     = numerator / max(denominator, 0.001);  
            
        // add to outgoing radiance Lo
        float NdotL = max(dot(N, L), 0.0);                
        Lo += (kD * albedo / PI + specular) * radiance * NdotL; 
    }   
  
    vec3 ambient = vec3(0.03) * albedo * ao;
    vec3 color = ambient + Lo;
	
    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0/2.2));  
   
    FragColor = vec4(color, 1.0);
}
";
*/
