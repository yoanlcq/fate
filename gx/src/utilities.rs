// Keep this mod lightweight.
use gl::{self, types::*};

pub fn gl_at_least(major: GLuint, minor: GLuint) -> bool {
    ::GLVersion::current().gl(major, minor)
}
pub fn gles_at_least(major: GLuint, minor: GLuint) -> bool {
    ::GLVersion::current().gles(major, minor)
}

pub fn init_reasonable_default_gl_state() {
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
        gl::GetError(); // Eat any errors caused by the line above
        gl::ClearColor(1., 0., 1., 1.);
    }
}

