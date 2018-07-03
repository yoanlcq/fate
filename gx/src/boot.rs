
pub fn boot_gl() {
    let ext = unsafe { &mut ::extensions::CACHE };
    *ext = Some(::ExtensionsStore::new().useful_extensions());
    let &mut ::UsefulExtensions {
        khr_debug,
        ati_meminfo,
        nvx_gpu_memory_info,
    } = ext.as_mut().unwrap();

    if khr_debug {
        ::debug::init_debug_output_khr();
    }
    if ati_meminfo {
        ::meminfo::init_ati();
    }
    if nvx_gpu_memory_info {
        ::meminfo::init_nvx();
    }
    ::init_reasonable_default_gl_state();
}
