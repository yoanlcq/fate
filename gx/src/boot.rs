
pub fn boot_gl() {
    let ext = unsafe { &mut ::extensions::CACHE };
    *ext = Some(::ExtensionsStore::new().useful_extensions());
    let &::UsefulExtensions {
        khr_debug,
        ati_meminfo,
        nvx_gpu_memory_info,
        arb_pipeline_statistics_query,
        arb_timer_query,
    } = ext.as_ref().unwrap();

    if khr_debug {
        ::debug::init_debug_output_khr();
    }
    if ati_meminfo {
        ::meminfo::init_ati();
    }
    if nvx_gpu_memory_info {
        ::meminfo::init_nvx();
    }
    if arb_pipeline_statistics_query {
        ::query::init_arb_pipeline_statistics_query();
    }
    if arb_timer_query {
        ::query::init_arb_timer_query();
    }
    ::init_reasonable_default_gl_state();
}
