use std::collections::HashSet;
use gl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionsStore(HashSet<String>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsefulExtensions {
    pub khr_debug: bool,
    pub ati_meminfo: bool,
    pub nvx_gpu_memory_info: bool,
    pub arb_pipeline_statistics_query: bool,
    pub arb_timer_query: bool,
}

pub static mut CACHE: Option<UsefulExtensions> = None;

impl ExtensionsStore {
    pub fn new() -> Self {
        let nb_extensions = ::integer(gl::NUM_EXTENSIONS);
        assert!(nb_extensions >= 0);
        ExtensionsStore((0..nb_extensions).map(|i| ::string_i(gl::EXTENSIONS, i as _)).collect())
    }
    pub fn has(&self, ext: &str) -> bool {
        self.0.contains(ext)
    }
    pub fn useful_extensions(&self) -> UsefulExtensions {
        let v = ::GLVersion::current();
        UsefulExtensions {
            khr_debug: v.gl(4, 3) || self.has("GL_KHR_debug") || self.has("GL_ARB_debug_output") /* ARB falls back to KHR with current gl crate */,
            ati_meminfo: self.has("GL_ATI_meminfo"),
            nvx_gpu_memory_info: self.has("GL_NVX_gpu_memory_info"),
            arb_pipeline_statistics_query: self.has("ARB_pipeline_statistics_query"),
            arb_timer_query: v.gl(3, 3) || self.has("ARB_timer_query"),
        }
    }
}

