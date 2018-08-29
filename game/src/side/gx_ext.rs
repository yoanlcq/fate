fn gx_buffer_data<T>(target: gx::BufferTarget, data: &[T], usage: gx::BufferUsage) {
    unsafe {
        gl::BufferData(target as _, mem::size_of_val(data) as _, data.as_ptr() as _, usage as _);
    }
}
fn gx_buffer_data_dsa<T>(buf: &gx::Buffer, data: &[T], usage: gx::BufferUsage) {
    unsafe {
        gl::BindBuffer(gx::BufferTarget::Array as _, buf.gl_id());
        gx_buffer_data(gx::BufferTarget::Array, data, usage);
        gl::BindBuffer(gx::BufferTarget::Array as _, 0);
    }
}


#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FramebufferStatus {
     UnknownError                = 0,
     Complete                    = gl::FRAMEBUFFER_COMPLETE,
     Undefined                   = gl::FRAMEBUFFER_UNDEFINED,
     IncompleteAttachment        = gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT,
     IncompleteMissingAttachment = gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT,
     IncompleteDrawBuffer        = gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER,
     IncompleteReadBuffer        = gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER,
     Unsupported                 = gl::FRAMEBUFFER_UNSUPPORTED,
     IncompleteMultisample       = gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE,
     IncompleteLayerTargets      = gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS,
}

impl FramebufferStatus {
    pub fn try_from_glenum(e: GLenum) -> Option<Self> {
        match e {
            0                                             => Some(FramebufferStatus::UnknownError               ),
            gl::FRAMEBUFFER_COMPLETE                      => Some(FramebufferStatus::Complete                   ),
            gl::FRAMEBUFFER_UNDEFINED                     => Some(FramebufferStatus::Undefined                  ),
            gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT         => Some(FramebufferStatus::IncompleteAttachment       ),
            gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => Some(FramebufferStatus::IncompleteMissingAttachment),
            gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER        => Some(FramebufferStatus::IncompleteDrawBuffer       ),
            gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER        => Some(FramebufferStatus::IncompleteReadBuffer       ),
            gl::FRAMEBUFFER_UNSUPPORTED                   => Some(FramebufferStatus::Unsupported                ),
            gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE        => Some(FramebufferStatus::IncompleteMultisample      ),
            gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS      => Some(FramebufferStatus::IncompleteLayerTargets     ),
            _ => None,
        }
    }
    pub fn is_complete(&self) -> bool {
        *self == FramebufferStatus::Complete
    }
    pub fn to_result(&self) -> Result<(), Self> {
        if self.is_complete() { Ok(()) } else { Err(*self) }
    }
}

fn init_gbuffer() {
    let gbuffer_texture_formats = [
        gl::RGB32F, gl::RGB32F, gl::RGB32F, gl::RGB32F, gl::RGBA32F, gl::DEPTH_COMPONENT32F
    ];
    let gbuffer_textures = [0, 0, 0, 0, 0, 0];
    let draw_buffers = [
        gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2, gl::COLOR_ATTACHMENT3, gl::COLOR_ATTACHMENT4
    ];


    let mut fbo = 0;
    unsafe {
        gl::GenFramebuffers(1, &mut fbo);
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, fbo);
        gl::FramebufferTexture2D(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, gbuffer_textures[0], 0);
        gl::FramebufferTexture2D(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, gbuffer_textures[1], 0);
        gl::FramebufferTexture2D(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT2, gl::TEXTURE_2D, gbuffer_textures[2], 0);
        gl::FramebufferTexture2D(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT3, gl::TEXTURE_2D, gbuffer_textures[3], 0);
        gl::FramebufferTexture2D(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT4, gl::TEXTURE_2D, gbuffer_textures[4], 0);
        gl::FramebufferTexture2D(gl::DRAW_FRAMEBUFFER, gl::DEPTH_ATTACHMENT,  gl::TEXTURE_2D, gbuffer_textures[5], 0);
        gl::DrawBuffers(draw_buffers.len() as _, draw_buffers.as_ptr());
        gl::CheckFramebufferStatus(gl::DRAW_FRAMEBUFFER);
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
    }

    /*
     * shader.vert
layout(location = 0) out vec3 fPosition;
layout(location = 1) out vec3 fNormal;
layout(location = 2) out vec3 fAmbient;
layout(location = 3) out vec3 fDiffuse;
layout(location = 4) out vec4 fGlossyShininess;
    */
}

fn draw_to_fbo(fbo: u32, w: i32, h: i32) {
    unsafe {
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, fbo);
        gl::Viewport(0, 0, w, h);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
    // render
}

// NOTE: Blitting also works with FBO 0 as destination (the screen!)
fn blit_fbos(src_fbo: u32, w: i32, h: i32) {
    unsafe {
        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, src_fbo);
        gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        gl::BlitFramebuffer(0, 0, 0, 0, w, h, w, h, gl::COLOR_BUFFER_BIT, gl::NEAREST);
    }
}

/*
fn cs() {
    gl::MAX_COMPUTE_WORK_GROUP_COUNT // Number of dispatches
    gl::MAX_COMPUTE_WORK_GROUP_SIZE // Individual max local_size in X, Y and Z 
    gl::MAX_COMPUTE_WORK_GROUP_INVOCATIONS // Max total product of local_size X, Y and Z.
/*
#ifdef NVIDIA
layout(local_size_x = 4, local_size_y = 4) in;
#elif defined AMD
layout(local_size_x = 8, local_size_y = 8) in;
#else
layout(local_size_x = 32, local_size_y = 32) in;
#endif
layout(rgba32f) uniform  readonly restrict image2D u_src;
                uniform writeonly restrict image2D u_dst;

// imageSize(u_src);
// gl_GlobalInvocationID.xy;
*/


    prog.set_uniform("u_src", 0);
    prog.set_uniform("u_dst", 1);
    gl::BindImageTexture(0, src_tex.gl_id(), 0, gl::FALSE, 0, gl::READ_ONLY , gl::RGBA32F);
    gl::BindImageTexture(1, dst_tex.gl_id(), 0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RGBA32F);
    // NOTE!!!! 32 = local_size dans le compute shader. 
    gl::DispatchCompute(1 + w / 32, 1 + h / 32, 1);
    gl::MemoryBarrier(gl::TEXTURE_FETCH_BARRIER_BIT | gl::SHADER_IMAGE_ACCESS_BARRIER_BIT | gl::FRAMEBUFFER_BARRIER_BIT);
}
*/
