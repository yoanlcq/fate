use std::fmt::{self, Display, Formatter};
use gl::{self, types::*};
use missing_bits;
use ::GLVersion;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ContextSummary {
    pub gl_version: GLVersion,
    pub gl_version_string: String,
    pub gl_renderer: String,
    pub gl_vendor: String,
    pub glsl_version: String,
    pub ctxflags: GLuint,
    pub ctxpmask: GLuint,
    pub depth_bits: GLuint,
    pub stencil_bits: GLuint,
    pub double_buffer: bool,
    pub stereo_buffers: bool,
}

impl ContextSummary {
    pub fn new() -> Self {
        let gl_version_string = ::gl_version_string();
        let gl_version = GLVersion::from_gl_version_string(&gl_version_string);
        Self {
            gl_version,
            gl_version_string,
            gl_renderer: ::gl_renderer_string(),
            gl_vendor: ::gl_vendor_string(),
            glsl_version: ::glsl_version_string(),
            ctxflags: ::context_flags(),
            ctxpmask: ::context_profile_mask(),
            depth_bits: ::depth_bits(),
            stencil_bits: ::stencil_bits(),
            double_buffer: ::doublebuffer(),
            stereo_buffers: ::stereo(),
        }
    }
}

impl Display for ContextSummary {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let &Self {
            ref gl_version,
            ref gl_version_string,
            ref gl_renderer,
            ref gl_vendor,
            ref glsl_version,
            ctxflags,
            ctxpmask,
            depth_bits,
            stencil_bits,
            double_buffer,
            stereo_buffers,
        } = self;

        write!(f,
"Version             : {} (parsed: {})
Renderer            : {}
Vendor              : {}
GLSL version        : {}
Profile flags       : {} (bits: 0b{:08b})
Context flags       : {}{}{}{} (bits: 0b{:08b})
Double buffering    : {}
Stereo buffers      : {}
Depth buffer bits   : {}
Stencil buffer bits : {}",
            gl_version_string, gl_version, gl_renderer, gl_vendor, glsl_version,
            if ctxpmask & gl::CONTEXT_CORE_PROFILE_BIT != 0 {
                "core"
            } else if ctxpmask & gl::CONTEXT_COMPATIBILITY_PROFILE_BIT != 0 {
                "compatibility"
            } else { "" },
            ctxpmask,
            if ctxflags & gl::CONTEXT_FLAG_FORWARD_COMPATIBLE_BIT != 0 { "forward_compatible " } else {""},
            if ctxflags & gl::CONTEXT_FLAG_DEBUG_BIT != 0 { "debug " } else {""},
            if ctxflags & gl::CONTEXT_FLAG_ROBUST_ACCESS_BIT != 0 { "robust_access " } else {""},
            if ctxflags & missing_bits::CONTEXT_FLAG_NO_ERROR_BIT_KHR != 0 { "no_error " } else {""},
            ctxflags,
            double_buffer, stereo_buffers, depth_bits, stencil_bits,
        )
    }
}


