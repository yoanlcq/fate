#[repr(u32)]
pub enum VAttrib {
    Position = 0,
    Normal = 1,
    Tangent = 2,
    BiTangent = 3,
    Color = 4,
    Uv = 5,
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
layout(location = 5) in vec4 a_texcoords;

out vec3 v_tex_coords;

void main() {
    v_tex_coords = a_position.xyz;
    vec4 pos = u_proj_matrix * u_modelview_matrix * vec4(a_position.xyz, 1.0);
    gl_Position = pos.xyww; // Z = 1 after perspective divide by w
}
";

static SKY_FS_SRC: &'static [u8] = b"
#version 450 core

struct TextureSelector {
    uint tab;
    float layer;
};

uniform samplerCubeArray u_cube_map_tabs[4]; // Solid 1x1, Low-res, Medium-res, Hi-res
uniform TextureSelector u_skybox;

in vec3 v_tex_coords;

out vec4 f_color;

void main() {
    f_color = texture(u_cube_map_tabs[u_skybox.tab], vec4(v_tex_coords, u_skybox.layer));
}
";

static TEXT_VS_SRC: &'static [u8] = b"
#version 450 core

uniform mat4 u_mvp;

layout(location = 0) in vec2 a_position;
layout(location = 5) in vec2 a_tex_coords;

out vec2 v_tex_coords;

void main() {
    v_tex_coords = a_tex_coords;
    gl_Position = u_mvp * vec4(a_position, 0.0, 1.0);
}
";

static TEXT_FS_SRC: &'static [u8] = b"
#version 450 core

uniform sampler2DArray u_atlas_array;
uniform float u_atlas_index;
uniform vec4 u_color;

in vec2 v_tex_coords;

out vec4 f_color;

void main() {
    float alpha = texture(u_atlas_array, vec3(v_tex_coords, u_atlas_index)).r;

    if (alpha <= 0.01)
        discard;

    f_color = vec4(u_color.rgb, u_color.a * alpha);
}
";


fn new_gl_color_program() -> Result<gx::ProgramEx, String> {
    let vs = gx::VertexShader::try_from_source(VS_SRC)?;
    let fs = gx::FragmentShader::try_from_source(FS_SRC)?;
    let prog = gx::Program::try_from_vert_frag(&vs, &fs)?;
    Ok(gx::ProgramEx::new(prog))
}
fn new_gl_skybox_program() -> Result<gx::ProgramEx, String> {
    let vs = gx::VertexShader::try_from_source(SKY_VS_SRC)?;
    let fs = gx::FragmentShader::try_from_source(SKY_FS_SRC)?;
    let prog = gx::Program::try_from_vert_frag(&vs, &fs)?;
    Ok(gx::ProgramEx::new(prog))
}
fn new_gl_text_program() -> Result<gx::ProgramEx, String> {
    let vs = gx::VertexShader::try_from_source(TEXT_VS_SRC)?;
    let fs = gx::FragmentShader::try_from_source(TEXT_FS_SRC)?;
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


