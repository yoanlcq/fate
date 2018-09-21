use fate::gx;

pub mod glsystem;
pub mod gl_setup;
pub mod gl_skybox;
pub mod gl_test_mdi_scene;

pub use self::glsystem::GLSystem;


fn unwrap_or_display_error(r: Result<gx::ProgramEx, String>) -> gx::ProgramEx {
    match r {
        Ok(p) => p,
        Err(e) => {
            error!("GL compile error\n{}", e);
            panic!("GL compile error\n{}", e)
        },
    }
}
fn new_program_ex(vs: &[u8], fs: &[u8]) -> Result<gx::ProgramEx, String> {
    let vs = gx::VertexShader::try_from_source(vs)?;
    let fs = gx::FragmentShader::try_from_source(fs)?;
    let prog = gx::Program::try_from_vert_frag(&vs, &fs)?;
    Ok(gx::ProgramEx::new(prog))
}
fn new_program_ex_unwrap(vs: &[u8], fs: &[u8]) -> gx::ProgramEx {
    unwrap_or_display_error(new_program_ex(vs, fs))
}