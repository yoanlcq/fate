use gl;

/// STREAM: The data store contents will be modified once and used at most a few times.
/// STATIC: The data store contents will be modified once and used many times.
/// DYNAMIC: The data store contents will be modified repeatedly and used many times.
/// 
/// DRAW: The data store contents are modified by the application, and used as the source for GL drawing and image specification commands.
/// READ: The data store contents are modified by reading data from the GL, and used to return that data when queried by the application.
/// COPY: The data store contents are modified by reading data from the GL, and used as the source for GL drawing and image specification commands.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BufferUsage {
    StreamDraw  = gl::STREAM_DRAW,
    StreamRead  = gl::STREAM_READ,
    StreamCopy  = gl::STREAM_COPY,
    StaticDraw  = gl::STATIC_DRAW,
    StaticRead  = gl::STATIC_READ,
    StaticCopy  = gl::STATIC_COPY,
    DynamicDraw = gl::DYNAMIC_DRAW,
    DynamicRead = gl::DYNAMIC_READ,
    DynamicCopy = gl::DYNAMIC_COPY,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BufferTarget {
    Array = gl::ARRAY_BUFFER,
    AtomicCounter = gl::ATOMIC_COUNTER_BUFFER,
    CopyRead = gl::COPY_READ_BUFFER,
    CopyWrite = gl::COPY_WRITE_BUFFER,
    DispatchIndirect = gl::DISPATCH_INDIRECT_BUFFER,
    DrawIndirect = gl::DRAW_INDIRECT_BUFFER,
    ElementArray = gl::ELEMENT_ARRAY_BUFFER,
    PixelPack = gl::PIXEL_PACK_BUFFER,
    PixelUnpack = gl::PIXEL_UNPACK_BUFFER,
    Query = gl::QUERY_BUFFER,
    ShaderStorage = gl::SHADER_STORAGE_BUFFER,
    Texture = gl::TEXTURE_BUFFER,
    TransformFeedback = gl::TRANSFORM_FEEDBACK_BUFFER,
    Uniform = gl::UNIFORM_BUFFER,
}
