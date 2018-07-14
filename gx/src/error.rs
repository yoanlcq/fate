use gl::{self, types::*};

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Error {
    InvalidEnum                 = gl::INVALID_ENUM,
    InvalidValue                = gl::INVALID_VALUE,
    InvalidOperation            = gl::INVALID_OPERATION,
    InvalidFramebufferOperation = gl::INVALID_FRAMEBUFFER_OPERATION,
    OutOfMemory                 = gl::OUT_OF_MEMORY,
    StackUnderflow              = gl::STACK_UNDERFLOW,
    StackOverflow               = gl::STACK_OVERFLOW,
}

impl Error {
    pub fn try_from_glenum(e: GLenum) -> Option<Self> {
        match e {
            gl::INVALID_ENUM                  => Some(Error::InvalidEnum                ),
            gl::INVALID_VALUE                 => Some(Error::InvalidValue               ),
            gl::INVALID_OPERATION             => Some(Error::InvalidOperation           ),
            gl::INVALID_FRAMEBUFFER_OPERATION => Some(Error::InvalidFramebufferOperation),
            gl::OUT_OF_MEMORY                 => Some(Error::OutOfMemory                ),
            gl::STACK_UNDERFLOW               => Some(Error::StackUnderflow             ),
            gl::STACK_OVERFLOW                => Some(Error::StackOverflow              ),
            _ => None,
        }
    }
    pub fn next() -> Option<Self> {
        match unsafe { gl::GetError() } {
            gl::NO_ERROR => None,
            e => Some(Self::try_from_glenum(e).unwrap()),
        }
    }
}

#[macro_export]
macro_rules! check_gl {
    () => { 
        check_gl!{"<no expression provided>"}
    };
    ($expr:expr) => {
        { let val = $expr; $crate::pump_gl_errors(stringify!($expr)); val }
    };
}

#[cfg(not(debug_assertions))]
pub fn pump_gl_errors(_: &str) {}
#[cfg(debug_assertions)]
pub fn pump_gl_errors(s: &str) {
    let error_hook = unsafe { ERROR_HOOK.expect("The GL error hook was not set") };
    while let Some(e) = Error::next() {
        (error_hook)(Some(e), s);
    }
    (error_hook)(None, s);
}

pub fn set_error_hook(f: fn(Option<Error>, &str)) {
    unsafe {
        ERROR_HOOK = Some(f);
    }
}

static mut ERROR_HOOK: Option<fn(Option<Error>, &str)> = None;
