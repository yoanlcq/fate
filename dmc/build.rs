// These are the OSes I plan to support. Any other is unlikely but 
// _might_ happen someday.
#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd", target_os = "netbsd"))]
static DISPLAY_BACKEND: &str = "x11";
#[cfg(target_os="windows")]
static DISPLAY_BACKEND: &str = "windows";
#[cfg(target_os="winrt")]
static DISPLAY_BACKEND: &str = "winrt";
#[cfg(target_os="android")]
static DISPLAY_BACKEND: &str = "android";
#[cfg(target_os="ios")]
static DISPLAY_BACKEND: &str = "uikit";
#[cfg(target_os="macos")]
static DISPLAY_BACKEND: &str = "appkit";
#[cfg(target_os="emscripten")]
static DISPLAY_BACKEND: &str = "emscripten";

fn main() {
    println!("cargo:rustc-cfg=dmc_display_backend=\"{}\"", DISPLAY_BACKEND);
}
