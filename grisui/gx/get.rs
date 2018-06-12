use std::ffi::CStr;
use gl;
use gl::types::*;

pub fn gl_version_string() -> String {
    unsafe {
        CStr::from_ptr(gl::GetString(gl::VERSION) as _).to_string_lossy().into()
    }
}
pub fn gl_renderer_string() -> String {
    unsafe {
        CStr::from_ptr(gl::GetString(gl::RENDERER) as _).to_string_lossy().into()
    }
}
pub fn gl_vendor_string() -> String {
    unsafe {
        CStr::from_ptr(gl::GetString(gl::VENDOR) as _).to_string_lossy().into()
    }
}
pub fn glsl_version_string() -> String {
    unsafe {
        CStr::from_ptr(gl::GetString(gl::SHADING_LANGUAGE_VERSION) as _).to_string_lossy().into()
    }
}
pub fn gl_extensions_string() -> String {
    unsafe {
        CStr::from_ptr(gl::GetString(gl::EXTENSIONS) as _).to_string_lossy().into()
    }
}

pub fn integer(x: GLenum) -> GLint {
    let mut i = 0;
    unsafe {
        gl::GetIntegerv(x, &mut i);
    }
    i
}
pub fn boolean(x: GLenum) -> bool {
    let mut i: GLboolean = 0;
    unsafe {
        gl::GetBooleanv(x, &mut i);
    }
    i == gl::TRUE as GLboolean
}


pub fn context_flags() -> GLuint { integer(gl::CONTEXT_FLAGS) as _ }
pub fn context_profile_mask() -> GLuint { integer(gl::CONTEXT_PROFILE_MASK) as _ }
pub fn stereo() -> bool { boolean(gl::STEREO) }
pub fn doublebuffer() -> bool { boolean(gl::DOUBLEBUFFER) }

pub fn depth_bits() -> GLuint {
    let mut depth_bits: GLint = 0;
    unsafe {
        gl::GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::DEPTH, 
                gl::FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE, &mut depth_bits);
    }
    depth_bits as _
}
pub fn stencil_bits() -> GLuint {
    let mut stencil_bits: GLint = 0;
    unsafe {
        gl::GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::STENCIL, 
                gl::FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE, &mut stencil_bits);
    }
    stencil_bits as _
}


