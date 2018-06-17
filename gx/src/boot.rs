pub fn boot_gl() {
    let ::UsefulExtensions {
        khr_debug,
    } = ::ExtensionsStore::new().useful_extensions();
    if khr_debug {
        ::debug::init_debug_output_khr();
    }
    ::init_reasonable_default_gl_state();
}
