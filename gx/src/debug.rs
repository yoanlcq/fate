use std::ptr;
use std::slice;
use std::os::raw::c_void;
use std::str;
use gl;
use gl::types::*;

pub fn init_debug_output_khr() {
    unsafe {
        SET_LABEL = set_label_real as _;

        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(gx_debug_message_callback, ptr::null_mut());
        gl::DebugMessageControl(
            gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE,
            0, ptr::null_mut(), gl::TRUE
        );
        log_debug_message("OpenGL debugging is set up.");
    }
}

pub fn log_debug_message(msg: &str) {
    let msg = msg.as_bytes();
    unsafe {
        gl::DebugMessageInsert(
            gl::DEBUG_SOURCE_APPLICATION, gl::DEBUG_TYPE_OTHER,
            0x00000000, gl::DEBUG_SEVERITY_NOTIFICATION,
            msg.len() as _, msg.as_ptr() as _
        );
    }
}

fn set_label_stub(_ns: ::Namespace, _id: GLuint, _label: &[u8]) {}
fn set_label_real(ns: ::Namespace, id: GLuint, label: &[u8]) {
    unsafe {
        gl::ObjectLabel(ns as _, id, label.len() as _, label.as_ptr() as _);
    }
}
static mut SET_LABEL: fn(::Namespace, GLuint, &[u8]) = set_label_stub;

pub fn set_label<T: ::Object>(o: &T, label: &[u8]) {
    let f = unsafe { SET_LABEL };
    (f)(T::NAMESPACE, o.gl_id(), label)
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DebugMessageSource {
    API = gl::DEBUG_SOURCE_API,
    WindowSystem = gl::DEBUG_SOURCE_WINDOW_SYSTEM,
    ShaderCompiler = gl::DEBUG_SOURCE_SHADER_COMPILER,
    ThirdParty = gl::DEBUG_SOURCE_THIRD_PARTY,
    Application = gl::DEBUG_SOURCE_APPLICATION,
    Other = gl::DEBUG_SOURCE_OTHER,
}

impl DebugMessageSource {
    pub fn try_from_glenum(i: GLenum) -> Option<Self> {
        match i {
            gl::DEBUG_SOURCE_API => Some(DebugMessageSource::API),
            gl::DEBUG_SOURCE_WINDOW_SYSTEM => Some(DebugMessageSource::WindowSystem),
            gl::DEBUG_SOURCE_SHADER_COMPILER => Some(DebugMessageSource::ShaderCompiler),
            gl::DEBUG_SOURCE_THIRD_PARTY => Some(DebugMessageSource::ThirdParty),
            gl::DEBUG_SOURCE_APPLICATION => Some(DebugMessageSource::Application),
            gl::DEBUG_SOURCE_OTHER => Some(DebugMessageSource::Other),
            _ => None,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DebugMessageType {
    Error = gl::DEBUG_TYPE_ERROR,
    DeprecatedBahaviour = gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR,
    UndefinedBehaviour = gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR,
    Performance = gl::DEBUG_TYPE_PERFORMANCE,
    Portability = gl::DEBUG_TYPE_PORTABILITY,
    Marker = gl::DEBUG_TYPE_MARKER,
    PushGroup = gl::DEBUG_TYPE_PUSH_GROUP,
    PopGroup = gl::DEBUG_TYPE_POP_GROUP,
    Other = gl::DEBUG_TYPE_OTHER,
}

impl DebugMessageType {
    pub fn try_from_glenum(i: GLenum) -> Option<Self> {
        match i {
            gl::DEBUG_TYPE_ERROR => Some(DebugMessageType::Error),
            gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => Some(DebugMessageType::DeprecatedBahaviour),
            gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => Some(DebugMessageType::UndefinedBehaviour),
            gl::DEBUG_TYPE_PERFORMANCE => Some(DebugMessageType::Performance),
            gl::DEBUG_TYPE_PORTABILITY => Some(DebugMessageType::Portability),
            gl::DEBUG_TYPE_MARKER => Some(DebugMessageType::Marker),
            gl::DEBUG_TYPE_PUSH_GROUP => Some(DebugMessageType::PushGroup),
            gl::DEBUG_TYPE_POP_GROUP => Some(DebugMessageType::PopGroup),
            gl::DEBUG_TYPE_OTHER => Some(DebugMessageType::Other),
            _ => None,
        }
    }
}


#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DebugMessageSeverity {
    High = gl::DEBUG_SEVERITY_HIGH,
    Medium = gl::DEBUG_SEVERITY_MEDIUM,
    Low = gl::DEBUG_SEVERITY_LOW,
    Notification = gl::DEBUG_SEVERITY_NOTIFICATION,
}

impl DebugMessageSeverity {
    pub fn try_from_glenum(i: GLenum) -> Option<Self> {
        match i { 
            gl::DEBUG_SEVERITY_HIGH         => Some(DebugMessageSeverity::High),
            gl::DEBUG_SEVERITY_MEDIUM       => Some(DebugMessageSeverity::Medium),
            gl::DEBUG_SEVERITY_LOW          => Some(DebugMessageSeverity::Low),
            gl::DEBUG_SEVERITY_NOTIFICATION => Some(DebugMessageSeverity::Notification),
            _ => None,
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DebugMessage<'a> {
    pub source: DebugMessageSource,
    pub type_: DebugMessageType,
    pub severity: DebugMessageSeverity,
    pub id: GLuint,
    pub text: &'a [u8],
}

pub type DebugMessageCallback = fn(&DebugMessage);

fn default_debug_message_callback(_: &DebugMessage) {}
static mut DEBUG_MESSAGE_CALLBACK: DebugMessageCallback = default_debug_message_callback;

pub fn set_debug_message_callback(f: Option<DebugMessageCallback>) {
    let f = f.unwrap_or(default_debug_message_callback);
    unsafe {
        DEBUG_MESSAGE_CALLBACK = f;
    }
}




extern "system" fn gx_debug_message_callback(
    source: GLenum, type_: GLenum, id: GLuint, severity: GLenum, 
    length: GLsizei, message: *const GLchar, _user_param: *mut c_void,
) {
    let f = unsafe { DEBUG_MESSAGE_CALLBACK };
    (f)(&DebugMessage {
        source: DebugMessageSource::try_from_glenum(source).unwrap(),
        type_: DebugMessageType::try_from_glenum(type_).unwrap(),
        severity: DebugMessageSeverity::try_from_glenum(severity).unwrap(),
        text: unsafe { slice::from_raw_parts(message as _, length as _) },
        id,
    })
}

