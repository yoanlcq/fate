use gl::{self, types::*};

pub const GL_VBO_FREE_MEMORY_ATI                         : GLenum = 0x87FB;
pub const GL_TEXTURE_FREE_MEMORY_ATI                     : GLenum = 0x87FC;
pub const GL_RENDERBUFFER_FREE_MEMORY_ATI                : GLenum = 0x87FD;
pub const GL_GPU_MEMORY_INFO_DEDICATED_VIDMEM_NVX        : GLenum = 0x9047;
pub const GL_GPU_MEMORY_INFO_TOTAL_AVAILABLE_MEMORY_NVX  : GLenum = 0x9048;
pub const GL_GPU_MEMORY_INFO_CURRENT_AVAILABLE_VIDMEM_NVX: GLenum = 0x9049;
pub const GL_GPU_MEMORY_INFO_EVICTION_COUNT_NVX          : GLenum = 0x904A;
pub const GL_GPU_MEMORY_INFO_EVICTED_MEMORY_NVX          : GLenum = 0x904B;

#[allow(non_upper_case_globals)]
static mut ATI_meminfo: bool = false;
#[allow(non_upper_case_globals)]
static mut NVX_gpu_memory_info: bool = false;

pub fn init_ati() {
    unsafe {
        ATI_meminfo = true;
    }
}
pub fn init_nvx() {
    unsafe {
        NVX_gpu_memory_info = true;
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct MemInfo {
    pub nvx: Option<MemInfoNVX>,
    pub ati: Option<MemInfoATI>,
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct MemInfoNVX {
    pub dedicated_vidmem_kilobytes: GLint,
    pub total_available_mem_kilobytes: GLint,
    pub current_available_vidmem_kilobytes: GLint,
    pub eviction_count: GLint,
    pub evicted_mem_kilobytes: GLint,
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct MemInfoATI {
    pub vbo_free_memory: FreeMemoryATI,
    pub texture_free_memory: FreeMemoryATI,
    pub renderbuffer_free_memory: FreeMemoryATI,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct FreeMemoryATI {
    pub total_kilobytes: GLint,
    pub largest_block_kilobytes: GLint,
    pub total_aux_kilobytes: GLint,
    pub largest_aux_block_kilobytes: GLint,
}

impl FreeMemoryATI {
    pub fn current(type_: GLenum) -> Option<Self> {
        if !unsafe { ATI_meminfo } {
            return None;
        }
        let mut s = Self::default();
        unsafe {
            gl::GetIntegerv(type_, &mut s.total_kilobytes); // first element
        }
        Some(s)
    }
}
impl MemInfoATI {
    pub fn current() -> Option<Self> {
        if !unsafe { ATI_meminfo } {
            return None;
        }
        Some(Self {
            vbo_free_memory: FreeMemoryATI::current(GL_VBO_FREE_MEMORY_ATI).unwrap(),
            texture_free_memory: FreeMemoryATI::current(GL_TEXTURE_FREE_MEMORY_ATI).unwrap(),
            renderbuffer_free_memory: FreeMemoryATI::current(GL_RENDERBUFFER_FREE_MEMORY_ATI).unwrap(),
        })
    }
}

impl MemInfoNVX {
    pub fn current() -> Option<Self> {
        if !unsafe { NVX_gpu_memory_info } {
            return None;
        }
        Some(Self {
            dedicated_vidmem_kilobytes: ::integer(GL_GPU_MEMORY_INFO_DEDICATED_VIDMEM_NVX),
            total_available_mem_kilobytes: ::integer(GL_GPU_MEMORY_INFO_TOTAL_AVAILABLE_MEMORY_NVX),
            current_available_vidmem_kilobytes: ::integer(GL_GPU_MEMORY_INFO_CURRENT_AVAILABLE_VIDMEM_NVX),
            eviction_count: ::integer(GL_GPU_MEMORY_INFO_EVICTION_COUNT_NVX),
            evicted_mem_kilobytes: ::integer(GL_GPU_MEMORY_INFO_EVICTED_MEMORY_NVX),
        })
    }
}

impl MemInfo {
    pub fn current() -> Self {
        Self { 
            nvx: MemInfoNVX::current(),
            ati: MemInfoATI::current(),
        }
    }
}
