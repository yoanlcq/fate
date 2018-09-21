use std::ptr;
use fate::gx::{self, {gl::{self, types::*}}};
use mesh::VertexAttribIndex;

#[derive(Debug)]
pub struct GLTestMDIScene {

}

impl GLTestMDIScene {
    pub fn new() -> Self {
        unimplemented!(); // The draw() method is evil
        Self {

        }
    }
    pub fn draw(&self) {
        unsafe {
            self.draw_unsafe()
        }
    }
    unsafe fn draw_unsafe(&self) {
        // Creating the resources
        
        let max_vertices = 0xffffff;
        let max_indices = 0xffffff;
        let max_instances = 0xffffff;

        let mut vao = 0;
        let mut buffers = [0; 6];

        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(buffers.len() as _, buffers.as_mut_ptr());

        let position_vbo = buffers[0];
        let normal_vbo = buffers[1];
        let uv_vbo = buffers[2];
        let model_matrix_vbo = buffers[3];
        let material_index_vbo = buffers[4];
        let ibo = buffers[5];

        let flags = gl::DYNAMIC_STORAGE_BIT;
        gl::NamedBufferStorage(position_vbo, max_vertices * 3 * 4, ptr::null(), flags);
        gl::NamedBufferStorage(normal_vbo, max_vertices * 3 * 4, ptr::null(), flags);
        gl::NamedBufferStorage(uv_vbo, max_vertices * 2 * 4, ptr::null(), flags);
        gl::NamedBufferStorage(model_matrix_vbo, max_instances * 4 * 4 * 4, ptr::null(), flags);
        gl::NamedBufferStorage(material_index_vbo, max_instances * 2, ptr::null(), flags);
        gl::NamedBufferStorage(ibo, max_indices * 4, ptr::null(), flags);

        // Specifying vertex attrib layout

        gl::BindVertexArray(vao);
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

        // TODO: Operations:
        // - Add mesh (grab chunks)
        // - Remove mesh (release chunks)
        // - Edit mesh (re-upload data)
        // - Add instance pack
        // - Remove instance pack
        // - Edit instance pack
        //
        // i.e
        // - Allocate N vertices
        // - Allocate N indices
        // - Allocate N instances
        // - Defragment the memory

        // Uploading data

        // gl::NamedBufferSubData(buf, offset, size, data);

        // Drawing

        let m = HeapInfo::default();
        let mut cmds = vec![];

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

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, 0); // read from cpu memory
        gl::MultiDrawElementsIndirect(gx::Topology::Triangles as _, gl::UNSIGNED_INT, cmds.as_ptr() as _, cmds.len() as _, 0);
        gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, 0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }
}

use std::ops::Range;

#[derive(Debug, Default)]
pub struct HeapInfo {
    // Indexed by mesh
    pub vertex_ranges: Vec<Range<u32>>,
    pub index_ranges: Vec<Range<u32>>,

    // Indexed by instance
    pub instance_ranges: Vec<Range<u32>>,
    pub instance_range_mesh_entry: Vec<u32>,
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(C)]
pub struct GLDrawElementsIndirectCommand {
    pub nb_indices: GLuint,
    pub nb_instances: GLuint,
    pub first_index: GLuint,
    pub base_vertex: GLuint,
    pub base_instance: GLuint,
}


static PBR_VS : &'static str = 
"#version 450 core

uniform mat4 u_viewproj_matrix;

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_normal;
layout(location = 5) in vec2 a_uv;
layout(location = 9) in mat4 a_model_matrix;
layout(location = 13) in uint a_material_index;

out vec3 v_position;
out vec3 v_normal;
out vec2 v_uv;
flat out uint v_material_index;

void main() {
    gl_Position = u_viewproj_matrix * a_model_matrix * vec4(a_position, 1.0);
    v_position = a_position;
    v_normal = a_normal;
    v_uv = a_uv;
    v_material_index = a_material_index;
}
";

// https://learnopengl.com/PBR/Lighting
static PBR_FS: &'static str = 
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
