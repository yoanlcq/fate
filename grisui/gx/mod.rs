pub mod get;
pub use self::get::*;
pub mod object;
pub use self::object::*;
pub mod buffer;
pub use self::buffer::*;
pub mod shader;
pub mod program;
pub mod texture_unit;
pub use self::texture_unit::*;

// TODO: report to gl crate.
pub mod missing_bits {
    use gl::types::*;
    pub const CONTEXT_FLAG_NO_ERROR_BIT_KHR: GLuint = 0x00000008;
}

pub fn parse_version_string(version_string: &str) -> (u32, u32) {
    (version_string.chars().nth(0).unwrap() as u32 - '0' as u32,
     version_string.chars().nth(2).unwrap() as u32 - '0' as u32)
}
