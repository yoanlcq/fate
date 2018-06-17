use std::collections::HashSet;
use gl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionsStore(HashSet<String>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsefulExtensions {
    pub khr_debug: bool,
}

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
        }
    }
}

