use std::fmt::{self, Display, Formatter};
use std::ptr;
use std::slice;
use std::os::raw::c_void;
use std::str;
use sdl2::video::{GLProfile};
use sdl2::video::gl_attr::GLAttr;
use gl;
use gl::types::*;
use gx;


pub fn configure_sdl2_gl_attr(gl_attr: GLAttr) {
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_flags().debug().set();
    //gl_attr.set_context_version(3, 2);
    gl_attr.set_depth_size(24);
    gl_attr.set_stencil_size(8);
    gl_attr.set_multisample_buffers(1);
    gl_attr.set_multisample_samples(4);
}

fn setup_gl_state() {
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        // Enable POINT_SPRITE for proprietary NVIDIA Linux drivers, otherwise:
        // - Points would be round by default (which is wrong; there are square);
        // - Point sprites just wouldn't work.
        gl::Enable(0x8861); // gl::POINT_SPRITE
        gl::ClearColor(1., 0., 1., 1.);
    }
}

pub fn boot_gl() {
    let summary = ContextSummary::default();
    info!("--- Active OpenGL context settings ---\n{}", &summary);
    if summary.has_khr_debug {
        setup_khr_debug_output();
    }
    setup_gl_state();
}

fn setup_khr_debug_output() {
    unsafe {
        SET_LABEL = set_label_real as _;

        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(grx_gl_dbg_msg_callback, ptr::null_mut());
        gl::DebugMessageControl(
            gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE,
            0, ptr::null_mut(), gl::TRUE
        );
        let msg = b"OpenGL debugging is set up.\0";
        gl::DebugMessageInsert(
            gl::DEBUG_SOURCE_APPLICATION, gl::DEBUG_TYPE_OTHER,
            0x00000000, gl::DEBUG_SEVERITY_NOTIFICATION,
            (msg.len()-1) as _, msg.as_ptr() as _
        );
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ContextSummary {
    pub gl_version: String,
    pub gl_renderer: String,
    pub gl_vendor: String,
    pub glsl_version: String,
    pub gl_major: u32,
    pub gl_minor: u32,
    pub ctxflags: GLuint,
    pub ctxpmask: GLuint,
    pub depth_bits: GLuint,
    pub stencil_bits: GLuint,
    pub double_buffer: bool,
    pub stereo_buffers: bool,
    pub has_khr_debug: bool,
}

impl Default for ContextSummary {
    fn default() -> Self {
        let gl_version = gx::gl_version_string();
        let (gl_major, gl_minor) = gx::parse_version_string(&gl_version);
        let gl_extensions = gx::gl_extensions_string();
        let has_khr_debug = gl_major > 4 || (gl_major == 4 && gl_minor >= 3)
            || gl_extensions.find("GL_KHR_debug").is_some();
        Self {
            gl_version,
            gl_renderer: gx::gl_renderer_string(),
            gl_vendor: gx::gl_vendor_string(),
            glsl_version: gx::glsl_version_string(),
            gl_major,
            gl_minor,
            ctxflags: gx::context_flags(),
            ctxpmask: gx::context_profile_mask(),
            depth_bits: gx::depth_bits(),
            stencil_bits: gx::stencil_bits(),
            double_buffer: gx::doublebuffer(),
            stereo_buffers: gx::stereo(),
            has_khr_debug,
        }
    }
}


impl Display for ContextSummary {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let &Self {
            ref gl_version,
            ref gl_renderer,
            ref gl_vendor,
            ref glsl_version,
            gl_major,
            gl_minor,
            ctxflags,
            ctxpmask,
            depth_bits,
            stencil_bits,
            double_buffer,
            stereo_buffers,
            has_khr_debug: _,
        } = self;

        writeln!(f,
"Version             : {} (parsed: {}.{})
Renderer            : {}
Vendor              : {}
GLSL version        : {}
Profile flags       : {} (bits: 0b{:08b})
Context flags       : {}{}{}{} (bits: 0b{:08b})
Double buffering    : {}
Stereo buffers      : {}
Depth buffer bits   : {}
Stencil buffer bits : {}",
            gl_version, gl_major, gl_minor, gl_renderer, gl_vendor, glsl_version,
            if ctxpmask & gl::CONTEXT_CORE_PROFILE_BIT != 0 {
                "core"
            } else if ctxpmask & gl::CONTEXT_COMPATIBILITY_PROFILE_BIT != 0 {
                "compatibility"
            } else { "" },
            ctxpmask,
            if ctxflags & gl::CONTEXT_FLAG_FORWARD_COMPATIBLE_BIT != 0 { "forward_compatible " } else {""},
            if ctxflags & gl::CONTEXT_FLAG_DEBUG_BIT != 0 { "debug " } else {""},
            if ctxflags & gl::CONTEXT_FLAG_ROBUST_ACCESS_BIT != 0 { "robust_access " } else {""},
            if ctxflags & gx::missing_bits::CONTEXT_FLAG_NO_ERROR_BIT_KHR != 0 { "no_error " } else {""},
            ctxflags,
            double_buffer, stereo_buffers, depth_bits, stencil_bits,
        )
    }
}

fn set_label_stub(_ns: gx::Namespace, _id: GLuint, _label: &[u8]) {}
fn set_label_real(ns: gx::Namespace, id: GLuint, label: &[u8]) {
    unsafe {
        gl::ObjectLabel(ns as _, id, label.len() as _, label.as_ptr() as _);
    }
}
static mut SET_LABEL: fn(gx::Namespace, GLuint, &[u8]) = set_label_stub;

pub fn set_label<T: gx::Object>(o: &T, label: &[u8]) {
    (unsafe { SET_LABEL })(T::NAMESPACE, o.gl_id(), label);
}


extern "system" fn grx_gl_dbg_msg_callback(
    source: GLenum, ty: GLenum, id: GLuint, severity: GLenum, 
    length: GLsizei, message: *const GLchar, _user_param: *mut c_void,
) {
    let src = match source {
        gl::DEBUG_SOURCE_API => "API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "Window system",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "Shader compiler",
        gl::DEBUG_SOURCE_THIRD_PARTY => "3rd party",
        gl::DEBUG_SOURCE_APPLICATION => "Application",
        gl::DEBUG_SOURCE_OTHER => "Other",
        _ => "",
    };
    use log::Level;
    let mut level = Level::Debug;
    let t = match ty {
        gl::DEBUG_TYPE_ERROR => { level = Level::Error; "Error" },
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => { level = Level::Warn; "Deprecated behaviour" },
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => { level = Level::Warn; "Undefined behaviour" },
        gl::DEBUG_TYPE_PERFORMANCE => "Performance",
        gl::DEBUG_TYPE_PORTABILITY => "Portability",
        gl::DEBUG_TYPE_MARKER => "Command stream annotation",
        gl::DEBUG_TYPE_PUSH_GROUP => "Push debug group",
        gl::DEBUG_TYPE_POP_GROUP => "Pop debug group",
        gl::DEBUG_TYPE_OTHER => "Other",
        _ => "",
    };
    let sev = match severity {
        gl::DEBUG_SEVERITY_HIGH         => "High",
        gl::DEBUG_SEVERITY_MEDIUM       => "Medium",
        gl::DEBUG_SEVERITY_LOW          => "Low",
        gl::DEBUG_SEVERITY_NOTIFICATION => "Info",
        _ => "",
    };
    let message = unsafe {
        slice::from_raw_parts(message as *const u8, length as _)
    };
    let message = str::from_utf8(message).unwrap();
    log!(
        level,
        "OpenGL debug message ({}, {}, {}, 0x{:X}) :\n{}",
        sev, t, src, id, message
    );
}

