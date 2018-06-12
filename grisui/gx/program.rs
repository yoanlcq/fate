use super::{Program, VertexShader, FragmentShader, Object};
use gl::{self, types::*};

impl Program {
    pub fn try_from_vert_frag(vs: &VertexShader, fs: &FragmentShader) -> Result<Self, String> {
        unsafe {
            let program = gl::CreateProgram();
            assert_ne!(program, 0);
            gl::AttachShader(program, vs.gl_id());
            gl::AttachShader(program, fs.gl_id());
            gl::LinkProgram(program);
            gl::DetachShader(program, vs.gl_id());
            gl::DetachShader(program, fs.gl_id());
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
            let s = Program(program);
            if status == gl::TRUE as _ {
                return Ok(s);
            }
            Err(s.info_log())
        }
    }
    pub fn info_log(&self) -> String {
        use ::std::ptr;
        unsafe {
            let mut len: GLint = 0;
            gl::GetProgramiv(self.gl_id(), gl::INFO_LOG_LENGTH, &mut len);
            let mut buf: Vec<u8> = Vec::with_capacity((len-1) as usize); // -1 to skip trailing null
            buf.set_len((len-1) as _);
            gl::GetProgramInfoLog(self.gl_id(), len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            String::from_utf8(buf).unwrap_or("<UTF-8 error>".to_owned())
        }
    }
    pub fn attrib_location(&self, name: &[u8]) -> Option<GLint> {
        assert_eq!(0, *name.last().unwrap());
        let i = unsafe {
            gl::GetAttribLocation(self.gl_id(), name.as_ptr() as *const GLchar)
        };
        match i {
            -1 => None,
            i @ _ => Some(i),
        }
    }
    pub fn uniform_location(&self, name: &[u8]) -> Option<GLint> {
        assert_eq!(0, *name.last().unwrap());
        let i = unsafe {
            gl::GetUniformLocation(self.gl_id(), name.as_ptr() as *const GLchar)
        };
        match i {
            -1 => None,
            i @ _ => Some(i),
        }
    }
    /*
    // WISH: Refactor this into a program Builer (do before linking)
    pub fn bind_attrib_location(&self, loc: GLuint, name: &[u8]) {
        assert_eq!(name[name.len()-1], 0);
        unsafe {
            gl::BindAttribLocation(self.gl_id(), loc, name.as_ptr() as *const GLchar);
        }
    }
    */

}
