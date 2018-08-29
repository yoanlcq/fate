
static mut NB_ERRORS: usize = 0;

pub fn gl_error_hook(e: Option<gx::Error>, s: &str) {
    match e {
        Some(e) => {
            error!("GL error: {:?} ({})", e, s);
            unsafe { NB_ERRORS += 1; }
        },
        None => if unsafe { NB_ERRORS > 0 } {
            panic!("Encountered {} OpenGL errors.", unsafe { NB_ERRORS });
        }
    }
}

pub fn gl_debug_message_callback(msg: &gx::DebugMessage) {
    match ::std::ffi::CString::new(msg.text) {
        Ok(cstr) => debug!("GL: {}", cstr.to_string_lossy()),
        Err(e) => debug!("GL (UTF-8 error): {}", e),
    };
}


