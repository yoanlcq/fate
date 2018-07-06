use std::ptr;
use gl;
use gl::types::*;
use super::object::*;

fn set_source(shader: GLuint, src: &[u8]) {
    unsafe {
        let mut len = src.len() as GLint;
        if src[len as usize - 1] == 0 {
            len -= 1;
        }
        let glchars = src.as_ptr() as *const GLchar;
        gl::ShaderSource(shader, 1, &glchars, &len);
    }
}
fn info_log_length(shader: GLuint) -> GLint {
    let mut len = 0;
    unsafe {
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
    }
    len
}

fn info_log(shader: GLuint) -> String {
    unsafe {
        let len = info_log_length(shader);
        let mut buf = Vec::<u8>::with_capacity((len-1) as _); // -1 to skip trailing null
        buf.set_len((len-1) as _);
        gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        String::from_utf8(buf).unwrap_or("<UTF-8 error>".to_owned())
    }
}

macro_rules! shader {
    ($Self:ident $ty:ident) => {
        impl $Self {
            pub fn try_from_source(src: &[u8]) -> Result<Self, String> {
                let s = Self::new();
                s.set_source(src);
                s.compile();
                match s.compile_status() {
                    Ok(()) => Ok(s),
                    Err(()) => Err(s.info_log()),
                }
            }
            pub fn compile_checked(&self) -> Result<(), String> {
                self.compile();
                match self.compile_status() {
                    Ok(()) => Ok(()),
                    Err(()) => Err(self.info_log()),
                }
            }
            pub fn set_source(&self, src: &[u8]) {
                set_source(self.0, src)
            }
            pub fn compile(&self) {
                unsafe {
                    gl::CompileShader(self.0);
                }
            }
            pub fn compile_status(&self) -> Result<(), ()> {
                let mut status = gl::FALSE as GLint;
                unsafe {
                    gl::GetShaderiv(self.gl_id(), gl::COMPILE_STATUS, &mut status);
                }
                if status == gl::TRUE as _ { Ok(()) } else { Err(()) }
            }
            pub fn info_log_length(&self) -> GLint {
                info_log_length(self.0)
            }
            pub fn info_log(&self) -> String {
                info_log(self.0)
            }
        }
    };
}

shader!{ ComputeShader        COMPUTE_SHADER         }
shader!{ VertexShader         VERTEX_SHADER          }
shader!{ TessControlShader    TESS_CONTROL_SHADER    }
shader!{ TessEvaluationShader TESS_EVALUATION_SHADER }
shader!{ GeometryShader       GEOMETRY_SHADER        }
shader!{ FragmentShader       FRAGMENT_SHADER        }

