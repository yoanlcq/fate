use gl::{self, types::*};
use ::{Object, Query};

// ARB_pipeline_statistics_query
pub const GL_VERTICES_SUBMITTED_ARB                : GLenum = 0x82EE;
pub const GL_PRIMITIVES_SUBMITTED_ARB              : GLenum = 0x82EF;
pub const GL_VERTEX_SHADER_INVOCATIONS_ARB         : GLenum = 0x82F0;
pub const GL_TESS_CONTROL_SHADER_PATCHES_ARB       : GLenum = 0x82F1;
pub const GL_TESS_EVALUATION_SHADER_INVOCATIONS_ARB: GLenum = 0x82F2;
pub const GL_GEOMETRY_SHADER_INVOCATIONS           : GLenum = 0x887F; // Yes, this one is not suffixed with _ARB
pub const GL_GEOMETRY_SHADER_PRIMITIVES_EMITTED_ARB: GLenum = 0x82F3;
pub const GL_FRAGMENT_SHADER_INVOCATIONS_ARB       : GLenum = 0x82F4;
pub const GL_COMPUTE_SHADER_INVOCATIONS_ARB        : GLenum = 0x82F5;
pub const GL_CLIPPING_INPUT_PRIMITIVES_ARB         : GLenum = 0x82F6;
pub const GL_CLIPPING_OUTPUT_PRIMITIVES_ARB        : GLenum = 0x82F7;


#[allow(non_upper_case_globals)]
static mut ARB_timer_query: bool = false;
#[allow(non_upper_case_globals)]
static mut ARB_pipeline_statistics_query: bool = false;


pub fn init_arb_pipeline_statistics_query() {
    unsafe {
        ARB_pipeline_statistics_query = true;
    }
}
pub fn init_arb_timer_query() {
    unsafe {
        ARB_timer_query = true;
    }
}


#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum QueryTarget {
    // ARB_timer_query
    TimeElapsed                         = gl::TIME_ELAPSED,
    // ARB_pipeline_statistics_query
    VerticesSubmittedARB                = GL_VERTICES_SUBMITTED_ARB, 
    PrimitivesSubmittedARB              = GL_PRIMITIVES_SUBMITTED_ARB,
    VertexShaderInvocationsARB          = GL_VERTEX_SHADER_INVOCATIONS_ARB,
    TessControlShaderPatchesARB         = GL_TESS_CONTROL_SHADER_PATCHES_ARB,
    TessEvaluationShaderInvocationsARB  = GL_TESS_EVALUATION_SHADER_INVOCATIONS_ARB,
    GeometryShaderInvocations           = GL_GEOMETRY_SHADER_INVOCATIONS,
    GeometryShaderPrimitivesEmittedARB  = GL_GEOMETRY_SHADER_PRIMITIVES_EMITTED_ARB,
    FragmentShaderInvocationsARB        = GL_FRAGMENT_SHADER_INVOCATIONS_ARB,
    ComputeShaderInvocationsARB         = GL_COMPUTE_SHADER_INVOCATIONS_ARB,
    ClippingInputPrimitivesARB          = GL_CLIPPING_INPUT_PRIMITIVES_ARB,
    ClippingOutputPrimitivesARB         = GL_CLIPPING_OUTPUT_PRIMITIVES_ARB,
}

impl QueryTarget {
    pub fn is_supported(&self) -> bool {
        match *self {
            QueryTarget::TimeElapsed => unsafe {
                ARB_timer_query
            },
            QueryTarget::VerticesSubmittedARB                |
            QueryTarget::PrimitivesSubmittedARB              |
            QueryTarget::VertexShaderInvocationsARB          |
            QueryTarget::TessControlShaderPatchesARB         |
            QueryTarget::TessEvaluationShaderInvocationsARB  |
            QueryTarget::GeometryShaderInvocations           |
            QueryTarget::GeometryShaderPrimitivesEmittedARB  |
            QueryTarget::FragmentShaderInvocationsARB        |
            QueryTarget::ComputeShaderInvocationsARB         |
            QueryTarget::ClippingInputPrimitivesARB          |
            QueryTarget::ClippingOutputPrimitivesARB         => unsafe {
                ARB_pipeline_statistics_query
            },
        }
    }
    pub fn counter_bits(&self) -> GLint {
        assert!(self.is_supported());
        let mut bits = 0;
        unsafe {
            gl::GetQueryiv(*self as _, gl::QUERY_COUNTER_BITS, &mut bits);
        }
        bits
    }
    pub fn begin(&self, query: &Query) {
        assert!(self.is_supported());
        unsafe {
            gl::BeginQuery(*self as _, query.gl_id());
        }
    }
    pub fn end(&self) {
        assert!(self.is_supported());
        unsafe {
            gl::EndQuery(*self as _);
        }
    }
}

impl Query {
    pub fn is_result_available(&self) -> bool {
        let mut yes = 0;
        unsafe {
            gl::GetQueryObjectuiv(self.gl_id(), gl::QUERY_RESULT_AVAILABLE, &mut yes);
        }
        yes != 0
    }
    pub fn wait_result(&self) -> u64 {
        let mut result = 0;
        unsafe {
            gl::GetQueryObjectui64v(self.gl_id(), gl::QUERY_RESULT, &mut result);
        }
        result
    }
}
