use std::ops::Range;
use std::os::raw::c_void;
use std::mem;
use std::ptr;
use std::marker::PhantomData;

use gl::{self, types::*};
use {Buffer, Object};

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

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum BufferAccess {
    ReadOnly = gl::READ_ONLY,
    WriteOnly = gl::WRITE_ONLY,
    ReadWrite = gl::READ_WRITE,
}

bitflags! {
    pub struct BufferFlags: GLbitfield {
        /// The contents of the data store may be updated after creation through calls to glBufferSubData. If this bit is not set, the buffer content may not be directly updated by the client. The data argument may be used to specify the initial content of the buffer's data store regardless of the presence of the GL_DYNAMIC_STORAGE_BIT. Regardless of the presence of this bit, buffers may always be updated with server-side calls such as glCopyBufferSubData and glClearBufferSubData.
        const DYNAMIC_STORAGE = gl::DYNAMIC_STORAGE_BIT;

        /// The data store may be mapped by the client for read access and a pointer in the client's address space obtained that may be read from.
        const MAP_READ = gl::MAP_READ_BIT;

        /// The data store may be mapped by the client for write access and a pointer in the client's address space obtained that may be written through.
        const MAP_WRITE = gl::MAP_WRITE_BIT;

        /// The client may request that the server read from or write to the buffer while it is mapped. The client's pointer to the data store remains valid so long as the data store is mapped, even during execution of drawing or dispatch commands.
        const MAP_PERSISTENT = gl::MAP_PERSISTENT_BIT;

        /// Shared access to buffers that are simultaneously mapped for client access and are used by the server will be coherent, so long as that mapping is performed using glMapBufferRange. That is, data written to the store by either the client or server will be immediately visible to the other with no further action taken by the application. In particular,
        ///
        /// - If GL_MAP_COHERENT_BIT is not set and the client performs a write followed by a call to the glMemoryBarrier command with the GL_CLIENT_MAPPED_BUFFER_BARRIER_BIT set, then in subsequent commands the server will see the writes.
        /// - If GL_MAP_COHERENT_BIT is set and the client performs a write, then in subsequent commands the server will see the writes.
        /// - If GL_MAP_COHERENT_BIT is not set and the server performs a write, the application must call glMemoryBarrier with the GL_CLIENT_MAPPED_BUFFER_BARRIER_BIT set and then call glFenceSync with GL_SYNC_GPU_COMMANDS_COMPLETE (or glFinish). Then the CPU will see the writes after the sync is complete.
        /// - If GL_MAP_COHERENT_BIT is set and the server does a write, the app must call glFenceSync with GL_SYNC_GPU_COMMANDS_COMPLETE (or glFinish). Then the CPU will see the writes after the sync is complete.
        const MAP_COHERENT = gl::MAP_COHERENT_BIT;

        /// When all other criteria for the buffer storage allocation are met, this bit may be used by an implementation to determine whether to use storage that is local to the server or to the client to serve as the backing store for the buffer.
        const CLIENT_STORAGE = gl::CLIENT_STORAGE_BIT;
    }
}

bitflags! {
    pub struct MapBufferRangeFlags: GLbitfield {
        /// The returned pointer may be used to read buffer object data. No GL error is generated if the pointer is used to query a mapping which excludes this flag, but the result is undefined and system errors (possibly including program termination) may occur.
        const READ = gl::MAP_READ_BIT;

        /// The returned pointer may be used to modify buffer object data. No GL error is generated if the pointer is used to modify a mapping which excludes this flag, but the result is undefined and system errors (possibly including program termination) may occur.
        const WRITE = gl::MAP_WRITE_BIT;

        /// The mapping is to be made in a persistent fassion and that the client intends to hold and use the returned pointer during subsequent GL operation. It is not an error to call drawing commands (render) while buffers are mapped using this flag. It is an error to specify this flag if the buffer's data store was not allocated through a call to the glBufferStorage command in which the GL_MAP_PERSISTENT_BIT was also set.
        const PERSISTENT = gl::MAP_PERSISTENT_BIT;

        /// Indicated that a persistent mapping is also to be coherent. Coherent maps guarantee that the effect of writes to a buffer's data store by either the client or server will eventually become visible to the other without further intervention from the application. In the absence of this bit, persistent mappings are not coherent and modified ranges of the buffer store must be explicitly communicated to the GL, either by unmapping the buffer, or through a call to glFlushMappedBufferRange or glMemoryBarrier.
        const COHERENT = gl::MAP_COHERENT_BIT; 

        /// The previous contents of the specified range may be discarded. Data within this range are undefined with the exception of subsequently written data. No GL error is generated if subsequent GL operations access unwritten data, but the result is undefined and system errors (possibly including program termination) may occur. This flag may not be used in combination with GL_MAP_READ_BIT.
        const INVALIDATE_RANGE = gl::MAP_INVALIDATE_RANGE_BIT;

        /// The previous contents of the entire buffer may be discarded. Data within the entire buffer are undefined with the exception of subsequently written data. No GL error is generated if subsequent GL operations access unwritten data, but the result is undefined and system errors (possibly including program termination) may occur. This flag may not be used in combination with GL_MAP_READ_BIT.
        const INVALIDATE_BUFFER = gl::MAP_INVALIDATE_BUFFER_BIT;

        /// One or more discrete subranges of the mapping may be modified. When this flag is set, modifications to each subrange must be explicitly flushed by calling glFlushMappedBufferRange. No GL error is set if a subrange of the mapping is modified and not flushed, but data within the corresponding subrange of the buffer are undefined. This flag may only be used in conjunction with GL_MAP_WRITE_BIT. When this option is selected, flushing is strictly limited to regions that are explicitly indicated with calls to glFlushMappedBufferRange prior to unmap; if this option is not selected glUnmapBuffer will automatically flush the entire mapped range when called.
        const FLUSH_EXPLICIT = gl::MAP_FLUSH_EXPLICIT_BIT;

        /// The GL should not attempt to synchronize pending operations on the buffer prior to returning from glMapBufferRange or glMapNamedBufferRange. No GL error is generated if pending operations which source or modify the buffer overlap the mapped region, but the result of such previous and any subsequent operations is undefined.
        const UNSYNCHRONIZED = gl::MAP_UNSYNCHRONIZED_BIT;
    }
}

impl BufferFlags {
    pub fn are_valid(&self) -> bool {
        if self.contains(Self::MAP_PERSISTENT) && !(self.contains(Self::MAP_READ) || self.contains(Self::MAP_WRITE)) {
            return false;
        }
        if self.contains(Self::MAP_COHERENT) && !self.contains(Self::MAP_PERSISTENT) {
            return false;
        }
        true
    }
}

impl BufferTarget {
    pub fn bind_buffer(&self, buf: &Buffer) {
        unsafe {
            gl::BindBuffer(*self as _, buf.gl_id());
        }
    }
    pub fn unbind_buffer(&self) {
        unsafe {
            gl::BindBuffer(*self as _, 0);
        }
    }
    pub fn set_buffer_data<T>(&self, data: &[T], buffer_usage: BufferUsage) {
        unsafe {
            gl::BufferData(*self as _, mem::size_of_val(data) as _, data.as_ptr() as _, buffer_usage as _);
        }
    }
    pub fn set_buffer_subdata<T>(&self, data: &[T], offset: usize) {
        unsafe {
            gl::BufferSubData(*self as _, offset as _, mem::size_of_val(data) as _, data.as_ptr() as _);
        }
    }
    pub fn copy_buffer_subdata_to(&self, dst: Self, src_offset: usize, dst_offset: usize, size: usize) {
        unsafe {
            gl::CopyBufferSubData(*self as _, dst as _, src_offset as _, dst_offset as _, size as _);
        }
    }
    pub fn set_buffer_storage<T>(&self, data: &[T], flags: BufferFlags) {
        assert!(flags.are_valid());
        unsafe {
            gl::BufferStorage(*self as _, mem::size_of_val(data) as _, data.as_ptr() as _, flags.bits());
        }
    }
    pub fn set_uninitialized_buffer_storage(&self, size: usize, flags: BufferFlags) {
        assert!(flags.are_valid());
        unsafe {
            gl::BufferStorage(*self as _, size as _, ptr::null(), flags.bits());
        }
    }
    pub fn flush_mapped_buffer_range(&self, range: Range<usize>) {
        assert!(range.start <= range.end);
        let length = range.end - range.start;
        unsafe {
            gl::FlushMappedBufferRange(*self as _, range.start as _, length as _);
        }
    }
    pub fn map_buffer(&self, access: BufferAccess) -> *mut c_void {
        unsafe {
            gl::MapBuffer(*self as _, access as _)
        }
    }
    pub fn unmap_buffer(&self) -> Result<(), ()> {
        unsafe {
            let ok = gl::UnmapBuffer(*self as _);
            if ok == gl::TRUE { Ok(()) } else { Err(()) }
        }
    }
    pub fn map_buffer_range(&self, range: Range<usize>, access: MapBufferRangeFlags) -> *mut c_void {
        assert!(range.start <= range.end);
        let length = range.end - range.start;
        unsafe {
            gl::MapBufferRange(*self as _, range.start as _, length as _, access.bits())
        }
    }
    pub fn buffer_parameter_iv(&self, param: GLenum) -> GLint {
        let mut val = 0;
        unsafe {
            gl::GetBufferParameteriv(*self as _, param, &mut val);
        }
        val
    }
    pub fn buffer_parameter_i64v(&self, param: GLenum) -> GLint64 {
        let mut val = 0;
        unsafe {
            gl::GetBufferParameteri64v(*self as _, param, &mut val);
        }
        val
    }
    pub fn buffer_access_flags(&self) -> Option<BufferAccess> {
        match self.buffer_parameter_iv(gl::BUFFER_ACCESS_FLAGS) {
            0 => None,
            f => Some(unsafe { mem::transmute(f) }),
        }
    }
    pub fn buffer_usage(&self) -> Option<BufferUsage> {
        match self.buffer_parameter_iv(gl::BUFFER_USAGE) {
            0 => None,
            f => Some(unsafe { mem::transmute(f) }),
        }
    }
    pub fn buffer_size(&self) -> usize { self.buffer_parameter_i64v(gl::BUFFER_SIZE) as _ }
    pub fn is_buffer_mapped(&self) -> bool { self.buffer_parameter_iv(gl::BUFFER_MAPPED) != 0 }
    pub fn buffer_map_offset(&self) -> usize { self.buffer_parameter_i64v(gl::BUFFER_MAP_OFFSET) as _ }
    pub fn buffer_map_length(&self) -> usize { self.buffer_parameter_i64v(gl::BUFFER_MAP_LENGTH) as _ }
    pub fn buffer_map_range(&self) -> Range<usize> { let start = self.buffer_map_offset(); start .. self.buffer_map_length() }
}

pub trait VertexIndex {
    const GL_TYPE: GLenum;
}

impl VertexIndex for u8  { const GL_TYPE: GLenum = gl::UNSIGNED_BYTE; }
impl VertexIndex for u16 { const GL_TYPE: GLenum = gl::UNSIGNED_SHORT; }
impl VertexIndex for u32 { const GL_TYPE: GLenum = gl::UNSIGNED_INT; }

// Features:
// - Capacity + bounds checking;
// - Typed/untyped duality;
// - API feels like DSA;
// - Optional CPU-side buffer;
// - Kinds (need to be known at creation time):
//   - Flex (BufferData)
//     - Usage: STATIC_, etc....
//     - Allows mapping
//   - Immutable (BufferStorage)
//     - Set data once (DYNAMIC_STORAGE)
//     - Allow mapping (Read | Write | Persistent | Coherent)
//       - If persistent, map when created, unmap when dropped.
//     - asserts that (len < capacity).
//
// Proposals:
// - OneshotBuffer (no DYNAMIC_STORAGE);
// - FlexBuffer (BufferData, Usage);
// - AzdoBuffer (Write | Persistent | Coherent + FenceSwapChain);
// > N.B: NOT paired with a CPU buffer. The CPU data may come from anywhere.
//   Also, splitting them is important to make sure we visually separate *building* the data on
//   the CPU versus *uploading* it to the GPU. Coupling these two saves some typing, but introduces
//   extra complexity and inefficiency in the API. The caller knows better.


#[derive(Debug, Hash, PartialEq, Eq)]
pub struct BufferEx<T> {
    inner: Buffer,
    _phantom_data: PhantomData<T>,
}

impl<T> BufferEx<T> {
    pub fn inner(&self) -> &Buffer {
        &self.inner
    }
    pub fn into_inner(self) -> Buffer {
        self.inner
    }
    pub fn new_flex(size: usize, usage: BufferUsage) -> Self {
        unimplemented!{}
    }
    pub fn new_storage(size: usize) -> Self {
        unimplemented!{}
    }
    pub fn capacity(&self) -> usize {
        unimplemented!{}
    }
    pub fn len(&self) -> usize {
        unimplemented!{}
    }
    pub fn clear(&self) {
        unimplemented!{}
    }
}

#[derive(Debug)]
pub struct FenceSwapChain {
    cpu_updates: bool,
    fences: [GLsync; 4], // Up to quadruple buffering
    capacity: u8,
    i: u8,
}

impl Drop for FenceSwapChain {
    fn drop(&mut self) {
        let &mut Self { cpu_updates: _, fences, capacity: _, i: _ } = self;
        for fence in fences.into_iter() {
            unsafe {
                gl::DeleteSync(*fence); // Tolerates zero
            }
        }
    }
}

fn toast() {
    let mut chain = FenceSwapChain::new_for_cpu_updates(3);
    loop {
        let _chunk_i = chain.start_frame();
        // memcpy(persistent_mapped_buffer.chunks()[chunk_i], cpu_mem...);
        // glDraw(....) // Using the buffer
        chain.end_frame();
    }
}

impl FenceSwapChain {
    pub fn new_for_cpu_updates(capacity: usize) -> Self {
        assert!(capacity <= 4);
        Self {
            cpu_updates: true,
            fences: [ptr::null_mut(); 4],
            capacity: capacity as u8,
            i: 0,
        }
    }
    // Returns a chunk index (index in N-buffered buffers)
    pub fn start_frame(&mut self) -> usize {
        let fence = &mut self.fences[self.i as usize];
        if !fence.is_null() {
            if self.cpu_updates {
                Self::wait_cpu(*fence);
            } else {
                Self::wait_gpu(*fence);
            }
            unsafe {
                gl::DeleteSync(*fence);
            }
            *fence = ptr::null_mut();
        }
        self.i as _
    }
    pub fn end_frame(&mut self) {
        {
            let fence = &mut self.fences[self.i as usize];
            assert!(fence.is_null());
            *fence = unsafe {
                gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0)
            };
            assert!(!fence.is_null());
        }
        self.i += 1;
        self.i %= self.capacity;
    }

    fn wait_gpu(sync: GLsync) {
        assert!(!sync.is_null());
        unsafe {
            gl::WaitSync(sync, 0, gl::TIMEOUT_IGNORED);
        }
    }
    fn wait_cpu(sync: GLsync) {
        assert!(!sync.is_null());
        let mut flags = 0;
        let mut timeout_nanos = 0;
        loop {
            match unsafe { gl::ClientWaitSync(sync, flags, timeout_nanos) } {
                gl::ALREADY_SIGNALED | gl::CONDITION_SATISFIED => break,
                gl::WAIT_FAILED => panic!(),
                _ => (),
            }
            flags = gl::SYNC_FLUSH_COMMANDS_BIT;
            timeout_nanos = 1_000_000_000; // 1 second. The timeout is not how long we'll actually wait, but a deadline before we give up.
        }
    }
}
