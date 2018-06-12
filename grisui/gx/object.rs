use gl;
use gl::types::*;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Namespace {
    Buffer            = gl::BUFFER,
    Shader            = gl::SHADER,
    Program           = gl::PROGRAM,
    VertexArray       = gl::VERTEX_ARRAY,
    Query             = gl::QUERY,
    ProgramPipeline   = gl::PROGRAM_PIPELINE,
    TransformFeedback = gl::TRANSFORM_FEEDBACK,
    Sampler           = gl::SAMPLER,
    Texture           = gl::TEXTURE,
    Renderbuffer      = gl::RENDERBUFFER,
    Framebuffer       = gl::FRAMEBUFFER,
}


pub trait Object: Sized {
    const NAMESPACE: Namespace;
    // Unsafe because it takes ownership of the raw GL object, which might not have been properly
    // set up.
    unsafe fn from_gl_id(id: GLuint) -> Self;
    fn gl_id(&self) -> GLuint;
    fn gl_create() -> GLuint;
    fn gl_gen(v: &mut [GLuint]);
    fn gl_delete_single(i: GLuint);
    fn gl_delete_multiple(v: &[GLuint]);
    fn to_option(&self) -> Option<&Self> {
        match self.gl_id() {
            0 => None,
            _ => Some(self),
        }
    }
    fn into_option(self) -> Option<Self> {
        match self.gl_id() {
            0 => { ::std::mem::forget(self); None },
            _ => Some(self),
        }
    }
}

macro_rules! object {
    ($Object:ident $Namespace:ident $single:ident $plural:ident) => {
        #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(C)]
        pub struct $Object(pub(super) GLuint);

        impl Default for $Object {
            fn default() -> Self {
                $Object(create::$single())
            }
        }
        impl Drop for $Object {
            fn drop(&mut self) {
                delete::$single(self.0);
            }
        }
        impl Object for $Object {
            const NAMESPACE: Namespace = Namespace::$Namespace;
            unsafe fn from_gl_id(id: GLuint) -> Self { $Object(id) }
            fn gl_id(&self) -> GLuint { self.0 }
            fn gl_create() -> GLuint { create::$single() }
            fn gl_gen(v: &mut [GLuint]) { gen::$plural(v); }
            fn gl_delete_single(i: GLuint) { delete::$single(i); }
            fn gl_delete_multiple(v: &[GLuint]) { delete::$plural(v); }
        }
        impl $Object {
            pub fn new() -> Self { Self::default() }
        }
    };
}

mod create {
    use super::*;
    pub fn program               () -> GLuint { unsafe { gl::CreateProgram() } }
    pub fn compute_shader        () -> GLuint { unsafe { gl::CreateShader(gl::COMPUTE_SHADER        ) } }
    pub fn vertex_shader         () -> GLuint { unsafe { gl::CreateShader(gl::VERTEX_SHADER         ) } }
    pub fn tess_control_shader   () -> GLuint { unsafe { gl::CreateShader(gl::TESS_CONTROL_SHADER   ) } }
    pub fn tess_evaluation_shader() -> GLuint { unsafe { gl::CreateShader(gl::TESS_EVALUATION_SHADER) } }
    pub fn geometry_shader       () -> GLuint { unsafe { gl::CreateShader(gl::GEOMETRY_SHADER       ) } }
    pub fn fragment_shader       () -> GLuint { unsafe { gl::CreateShader(gl::FRAGMENT_SHADER       ) } }

    pub fn buffer             () -> GLuint { let mut i = 0; unsafe { gl::GenBuffers           (1, &mut i); } i }
    pub fn vertex_array       () -> GLuint { let mut i = 0; unsafe { gl::GenVertexArrays      (1, &mut i); } i }
    pub fn query              () -> GLuint { let mut i = 0; unsafe { gl::GenQueries           (1, &mut i); } i }
    pub fn program_pipeline   () -> GLuint { let mut i = 0; unsafe { gl::GenProgramPipelines  (1, &mut i); } i }
    pub fn transform_feedback () -> GLuint { let mut i = 0; unsafe { gl::GenTransformFeedbacks(1, &mut i); } i }
    pub fn sampler            () -> GLuint { let mut i = 0; unsafe { gl::GenSamplers          (1, &mut i); } i }
    pub fn texture            () -> GLuint { let mut i = 0; unsafe { gl::GenTextures          (1, &mut i); } i }
    pub fn renderbuffer       () -> GLuint { let mut i = 0; unsafe { gl::GenRenderbuffers     (1, &mut i); } i }
    pub fn framebuffer        () -> GLuint { let mut i = 0; unsafe { gl::GenFramebuffers      (1, &mut i); } i }
}

mod gen {
    use super::*;
    pub fn programs               (v: &mut [GLuint]) { for p in v { *p = create::program               (); } }
    pub fn compute_shaders        (v: &mut [GLuint]) { for p in v { *p = create::compute_shader        (); } }
    pub fn vertex_shaders         (v: &mut [GLuint]) { for p in v { *p = create::vertex_shader         (); } }
    pub fn tess_control_shaders   (v: &mut [GLuint]) { for p in v { *p = create::tess_control_shader   (); } }
    pub fn tess_evaluation_shaders(v: &mut [GLuint]) { for p in v { *p = create::tess_evaluation_shader(); } }
    pub fn geometry_shaders       (v: &mut [GLuint]) { for p in v { *p = create::geometry_shader       (); } }
    pub fn fragment_shaders       (v: &mut [GLuint]) { for p in v { *p = create::fragment_shader       (); } }

    pub fn buffers             (v: &mut [GLuint]) { unsafe { gl::GenBuffers           (v.len() as _, v.as_mut_ptr()); } }
    pub fn vertex_arrays       (v: &mut [GLuint]) { unsafe { gl::GenVertexArrays      (v.len() as _, v.as_mut_ptr()); } }
    pub fn queries             (v: &mut [GLuint]) { unsafe { gl::GenQueries           (v.len() as _, v.as_mut_ptr()); } }
    pub fn program_pipelines   (v: &mut [GLuint]) { unsafe { gl::GenProgramPipelines  (v.len() as _, v.as_mut_ptr()); } }
    pub fn transform_feedbacks (v: &mut [GLuint]) { unsafe { gl::GenTransformFeedbacks(v.len() as _, v.as_mut_ptr()); } }
    pub fn samplers            (v: &mut [GLuint]) { unsafe { gl::GenSamplers          (v.len() as _, v.as_mut_ptr()); } }
    pub fn textures            (v: &mut [GLuint]) { unsafe { gl::GenTextures          (v.len() as _, v.as_mut_ptr()); } }
    pub fn renderbuffers       (v: &mut [GLuint]) { unsafe { gl::GenRenderbuffers     (v.len() as _, v.as_mut_ptr()); } }
    pub fn framebuffers        (v: &mut [GLuint]) { unsafe { gl::GenFramebuffers      (v.len() as _, v.as_mut_ptr()); } }
}

mod delete {
    use super::*;
    pub fn programs(v: &[GLuint]) { for p in v { program(*p); } }
    pub fn shaders (v: &[GLuint]) { for p in v { shader(*p); } }
    pub fn compute_shaders        (v: &[GLuint]) { shaders(v); }
    pub fn vertex_shaders         (v: &[GLuint]) { shaders(v); }
    pub fn tess_control_shaders   (v: &[GLuint]) { shaders(v); }
    pub fn tess_evaluation_shaders(v: &[GLuint]) { shaders(v); }
    pub fn geometry_shaders       (v: &[GLuint]) { shaders(v); }
    pub fn fragment_shaders       (v: &[GLuint]) { shaders(v); }

    pub fn program(i: GLuint) { unsafe { gl::DeleteProgram(i); } }
    pub fn shader (i: GLuint) { unsafe { gl::DeleteShader(i); } }
    pub fn compute_shader        (i: GLuint) { shader(i); }
    pub fn vertex_shader         (i: GLuint) { shader(i); }
    pub fn tess_control_shader   (i: GLuint) { shader(i); }
    pub fn tess_evaluation_shader(i: GLuint) { shader(i); }
    pub fn geometry_shader       (i: GLuint) { shader(i); }
    pub fn fragment_shader       (i: GLuint) { shader(i); }

    pub fn buffer             (mut i: GLuint) { unsafe { gl::DeleteBuffers           (1, &mut i); } }
    pub fn vertex_array       (mut i: GLuint) { unsafe { gl::DeleteVertexArrays      (1, &mut i); } }
    pub fn query              (mut i: GLuint) { unsafe { gl::DeleteQueries           (1, &mut i); } }
    pub fn program_pipeline   (mut i: GLuint) { unsafe { gl::DeleteProgramPipelines  (1, &mut i); } }
    pub fn transform_feedback (mut i: GLuint) { unsafe { gl::DeleteTransformFeedbacks(1, &mut i); } }
    pub fn sampler            (mut i: GLuint) { unsafe { gl::DeleteSamplers          (1, &mut i); } }
    pub fn texture            (mut i: GLuint) { unsafe { gl::DeleteTextures          (1, &mut i); } }
    pub fn renderbuffer       (mut i: GLuint) { unsafe { gl::DeleteRenderbuffers     (1, &mut i); } }
    pub fn framebuffer        (mut i: GLuint) { unsafe { gl::DeleteFramebuffers      (1, &mut i); } }

    pub fn buffers             (v: &[GLuint]) { unsafe { gl::DeleteBuffers           (v.len() as _, v.as_ptr()); } }
    pub fn vertex_arrays       (v: &[GLuint]) { unsafe { gl::DeleteVertexArrays      (v.len() as _, v.as_ptr()); } }
    pub fn queries             (v: &[GLuint]) { unsafe { gl::DeleteQueries           (v.len() as _, v.as_ptr()); } }
    pub fn program_pipelines   (v: &[GLuint]) { unsafe { gl::DeleteProgramPipelines  (v.len() as _, v.as_ptr()); } }
    pub fn transform_feedbacks (v: &[GLuint]) { unsafe { gl::DeleteTransformFeedbacks(v.len() as _, v.as_ptr()); } }
    pub fn samplers            (v: &[GLuint]) { unsafe { gl::DeleteSamplers          (v.len() as _, v.as_ptr()); } }
    pub fn textures            (v: &[GLuint]) { unsafe { gl::DeleteTextures          (v.len() as _, v.as_ptr()); } }
    pub fn renderbuffers       (v: &[GLuint]) { unsafe { gl::DeleteRenderbuffers     (v.len() as _, v.as_ptr()); } }
    pub fn framebuffers        (v: &[GLuint]) { unsafe { gl::DeleteFramebuffers      (v.len() as _, v.as_ptr()); } }
}


object!{ Program              Program            program                 programs               }
object!{ ComputeShader        Shader             compute_shader          compute_shaders        }
object!{ VertexShader         Shader             vertex_shader           vertex_shaders         }
object!{ TessControlShader    Shader             tess_control_shader     tess_control_shaders   }
object!{ TessEvaluationShader Shader             tess_evaluation_shader  tess_evaluation_shaders}
object!{ GeometryShader       Shader             geometry_shader         geometry_shaders       }
object!{ FragmentShader       Shader             fragment_shader         fragment_shaders       }
object!{ Buffer               Buffer             buffer                  buffers                }
object!{ VertexArray          VertexArray        vertex_array            vertex_arrays          }
object!{ Query                Query              query                   queries                }
object!{ ProgramPipeline      ProgramPipeline    program_pipeline        program_pipelines      }
object!{ TransformFeedback    TransformFeedback  transform_feedback      transform_feedbacks    }
object!{ Sampler              Sampler            sampler                 samplers               }
object!{ Texture              Texture            texture                 textures               }
object!{ Renderbuffer         Renderbuffer       renderbuffer            renderbuffers          }
object!{ Framebuffer          Framebuffer        framebuffer             framebuffers           }
 
