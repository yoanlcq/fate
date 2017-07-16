// Useful links and resources :
// - Well-written docs for X11
//   https://tronche.com/gui/x/
// - Extended Window Manager Hints
//   https://specifications.freedesktop.org/wm-spec/wm-spec-latest.html
// - Xplain on the XComposite extension
//   https://magcius.github.io/xplain/article/composite.html
//   https://cgit.freedesktop.org/xorg/proto/compositeproto/tree/compositeproto.txt
// - Translucent Windows in X
//   https://keithp.com/~keithp/talks/KeithPackardAls2000/
// - Clipboard wiki
//   https://www.freedesktop.org/wiki/Specifications/ClipboardsWiki/
// - XDND
//   https://www.freedesktop.org/wiki/Specifications/XDND/
// - GLX 1.4 spec
//   https://www.khronos.org/registry/OpenGL/specs/gl/glx1.4.pdf
// - GLX extensions
//   https://www.khronos.org/registry/OpenGL/index_gl.php
//
// Missing features:
// - Copy contexts
// - Share contexts (at glXCreateContext)
// - Destroy contexts
// - Off-screen rendering
//
// TODO: Talk a bit about how X11 UsefulAtoms are like Keys or Values for 
// Properties.
//
// The plan:
// - Display::open() chooses a screen (for use other than XDefaultScreen)
// - Get the root window for the screen with XRootWindow;
// - Display::open() calls glXQueryExtension and glXQueryVersion.
// - Display::open() calls glXGetClientString and glXQueryServerString
//   and glXQueryExtensionsString;
//
// Then, depending one the GLX version:
//
// - All versions :
//   - Always use glXGetProcAddressARB (glXGetProcAddress is not always 
//     exported);
//   - GLX_ARB_multisample
//   - GLX_EXT_swap_control
//   - GLX_EXT_swap_control_tear
//   - GLX_MESA_swap_control
//   - GLX_SGI_swap_control
// - 1.1
//   - glxChooseVisual (log glXGetConfig) + glXCreateContext;
// - 1.3
//   - glXChooseFBConfig (log glXGetFBConfigAttrib) + glxCreateNewContext;
// - 1.4
//   - GLX_SAMPLE_BUFFERS, GLX_SAMPLES (formerly ext GLX_ARB_multisample)
//   - try glXCreateContextAttribsARB, otherwise same as 1.3;
//   - GLX_CONTEXT_ROBUST_ACCESS_BIT_ARB
//   - GLX_EXT_create_context_es_profile
//   - GLX_EXT_create_context_es2_profile
// Then :
// - Log glXIsDirect()


extern crate x11;
use self::x11::xlib as x;
use self::x11::glx;

// Extra definitions that the x11 crate doesn't have
// (yet ? the crate version is 2.14.0 right now)
mod xx {
    #[no_implicit_prelude]
    pub const None: i32 = 0;

    pub const GLX_VENDOR: i32 = 1;
    pub const GLX_VERSION: i32 = 2;
    pub const GLX_EXTENSIONS: i32 = 3;

    use super::*;

    // See https://dri.freedesktop.org/wiki/glXGetProcAddressNeverReturnsNULL/
    // for a rationale.
    // Worth noting that it mentions Linux, but Linux is not the only OS
    // which X11 can run on.
    // SDL2 settles this by calling dlopen(DEFAULT_OPENGL), given:
    //     #if defined(__IRIX__)
    //     /* IRIX doesn't have a GL library versioning system */
    //     #define DEFAULT_OPENGL  "libGL.so"
    //     #elif defined(__MACOSX__)
    //     #define DEFAULT_OPENGL  "/usr/X11R6/lib/libGL.1.dylib"
    //     #elif defined(__QNXNTO__)
    //     #define DEFAULT_OPENGL  "libGL.so.3"
    //     #else
    //     #define DEFAULT_OPENGL  "libGL.so.1"
    //     #endif
    // then a bunch of glX functions are loaded from it.
    // 
    // But it's 2017 now, so I _really_ don't want to handle the 
    // 0.0001% of people whose libGL doesn't export GLX 1.4 functions 
    // (it's been around since 2011).
    // At some point you've gotta be sane.
    //
    // This _might_ cause issues later, but then it's the x11 crate which
    // would require some adjustments (i.e features to disable linking against
    // GLX 1.4 APIs).
    // OR define function types and load them all ourself.
    //
    // extern {
    //     pub fn glXGetProcAddressARB(name: *const c_uchar) -> Option<unsafe extern fn()>;
    // }
}

use std::fmt::{self, Formatter};
use std::ptr;
use std::mem;
use std::ffi::*;
use std::os::raw::{c_void, c_char, c_int};

use Extent2;

use super::{
    Semver,
    GLSwapInterval,
    GLContextSettings,
    Settings,
    Icon,
    Style,
};
use super::window::{
    Capabilities,
    WindowOpResult,
};




#[derive(Debug, Clone)]
pub enum Error {
    NoXDisplayForName { name: Option<CString> },
    NoGLX,
    UnsupportedGLContextSettings,
    FunctionName(&'static str),
}


impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            &Error::NoXDisplayForName { ref name } => {
                let name = unsafe { CString::from_raw(x::XDisplayName(
                    match name {
                        &None => ptr::null(),
                        &Some(ref name) => &name as *const _ as *const i8,
                    }
                ))};
                let name = name.to_str().unwrap_or("<utf-8 conversion error>");
                write!(fmt, "\"{}\" is not a valid X display", name)
            },
            &Error::NoGLX => {
                write!(fmt, "The GLX extension is not present")
            },
            &Error::UnsupportedGLContextSettings => {
                write!(fmt, "Unsupported OpenGL context settings")
            },
            &Error::FunctionName(name) => {
                write!(fmt, "{}() failed", name)
            },
        }
    }
}




// DISPLAY





/// Generate this module's `UsefulAtoms` struct, where all atoms are retrieved
/// at once when opening a display.
macro_rules! atoms {
    ($($atom:ident)+) => {
        #[allow(non_snake_case)]
        #[derive(Debug, Hash, PartialEq, Eq)]
        pub struct UsefulAtoms {
            $(pub $atom: x::Atom,)+
        }
        impl UsefulAtoms {
            fn fetch(x_dpy: *mut x::Display) -> Self {
                unsafe { Self { $(
                    $atom: x::XInternAtom(x_dpy, 
                        // PERF: Worried about CString
                        CString::new(stringify!($atom)).unwrap().as_ptr(), 
                        x::False),
                )+ } }
            }
        }
    }
}

atoms!(
    WM_PROTOCOLS
    WM_DELETE_WINDOW
    WM_TAKE_FOCUS
    _NET_NUMBER_OF_DESKTOPS
    _NET_CURRENT_DESKTOP
    _NET_DESKTOP_NAMES
    _NET_DESKTOP_VIEWPORT
    _NET_DESKTOP_GEOMETRY
    _NET_ACTIVE_WINDOW
    _NET_WM_NAME
    _NET_WM_ICON_NAME
    _NET_WM_WINDOW_TYPE
    _NET_WM_WINDOW_TYPE_DESKTOP
    _NET_WM_WINDOW_TYPE_SPLASH
    _NET_WM_WINDOW_TYPE_DIALOG
    _NET_WM_WINDOW_TYPE_NOTIFICATION
    // and others.. ??

    _NET_WM_STATE
    _NET_WM_STATE_HIDDEN
    _NET_WM_STATE_FOCUSED
    _NET_WM_STATE_MAXIMIZED_VERT
    _NET_WM_STATE_MAXIMIZED_HORZ
    _NET_WM_STATE_FULLSCREEN
    _NET_WM_STATE_ABOVE
    _NET_WM_STATE_SKIP_TASKBAR
    _NET_WM_STATE_SKIP_PAGER
    _NET_WM_STATE_DEMANDS_ATTENTION

    _NET_WM_ALLOWED_ACTIONS
    _NET_WM_ACTION_FULLSCREEN

    // This is an array of 32bit packed CARDINAL ARGB with high byte being A, low byte being B. The first two cardinals are width, height. Data is in rows, left to right and top to bottom.
    _NET_WM_ICON

    _NET_WM_PID

    // Should set this when going off-screen.
    _NET_WM_BYPASS_COMPOSITOR

    _NET_FRAME_EXTENTS
    _NET_WM_PING
    _NET_WM_WINDOW_OPACITY // Doesn't seem to be defined officially ??
    UTF8_STRING
    PRIMARY

    // X drag'n Drop atoms
    XdndEnter
    XdndPosition
    XdndStatus
    XdndTypeList
    XdndActionCopy
    XdndDrop
    XdndFinished
    XdndSelection

    // ??? from SDL2
    XKLAVIER_STATE
);

mod types {
    #![allow(non_camel_case_types)]
    use super::*;
    pub type glXGetProcAddress = unsafe extern fn(*const u8) -> Option<unsafe extern fn()>;
    pub type glXSwapIntervalMESA = unsafe extern fn(interval: c_int) -> c_int;
    pub type glXGetSwapIntervalMESA = unsafe extern fn() -> c_int;
    pub type glXSwapIntervalSGI = unsafe extern fn(interval: c_int) -> c_int;
    pub type glXSwapIntervalEXT = unsafe extern fn(
        *mut x::Display, glx::GLXDrawable, interval: c_int
    );
    pub type glXCreateContextAttribsARB = unsafe extern fn(
        *mut x::Display, glx::GLXFBConfig, share_context: glx::GLXContext, 
        direct: x::Bool, attrib_list: *const c_int) -> glx::GLXContext;
}

macro_rules! glx_ext {
    (($($name:ident)+) ($($func:ident)+)) => {
        #[allow(non_snake_case)]
        #[derive(Debug, Copy, Clone, Default, Hash, PartialEq, Eq)]
        pub struct GlxExt {
            $(pub $name: bool,)+
            $(pub $func: Option<types::$func>,)+
        }
        impl GlxExt {
            #[allow(non_snake_case)]
            fn parse(gpa: types::glXGetProcAddress, s: &CStr) -> Self {
                $(let mut $name = false;)+
                let s = s.to_string_lossy();
                for name in s.split_whitespace() {
                    match name {
                        $(stringify!($name) => $name = true,)+
                        _ => {}
                    };
                }
                let mut out = Self { $($name,)+ $($func: None,)+ };
                out.load_functions(gpa);
                out
            }
            fn load_functions(&mut self, gpa: types::glXGetProcAddress) {
                unsafe { $(
                let cstring = CString::new(stringify!($func)).unwrap_or_default();
                let name = cstring.to_bytes_with_nul();
                let fptr = gpa(name.as_ptr() as *mut _);
                self.$func = match fptr {
                    None => None,
                    Some(f) => Some(mem::transmute(f)),
                };
                )+ }
            }
        }
    }
}


glx_ext!((
    GLX_ARB_multisample
    GLX_EXT_swap_control
    GLX_EXT_swap_control_tear
    GLX_MESA_swap_control
    GLX_SGI_swap_control
    GLX_ARB_create_context
    GLX_ARB_create_context_profile
    GLX_ARB_create_context_robustness
    GLX_EXT_create_context_es_profile
    GLX_EXT_create_context_es2_profile
    )(
    glXSwapIntervalMESA
    glXGetSwapIntervalMESA
    glXSwapIntervalSGI
    glXSwapIntervalEXT
    glXCreateContextAttribsARB
));

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Glx {
    version: Semver,
    get_proc_address: types::glXGetProcAddress,
    ext: GlxExt,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Display {
    pub x_dpy: *mut x::Display,
    pub atoms: UsefulAtoms,
    // XXX Needs not be freed or not ? Seems not, but...
    pub screen: *mut x::Screen,
    pub screen_num: c_int,
    pub root: x::Window,
    pub glx: Option<Glx>,
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            x::XCloseDisplay(self.x_dpy);
        } 
    }
}

impl Display {

    pub(super) fn open() -> Result<Self,super::Error> {
        Self::open_x11_display_name(None)
    }

    pub(super) fn open_x11_display_name(x_display_name: Option<&CStr>) 
        -> Result<Self, super::Error> 
    {
        unsafe {
            let x_dpy = x::XOpenDisplay(match x_display_name {
                Some(s) => s.as_ptr(),
                None => ptr::null()
            });
            if x_dpy.is_null() {
                return Err(super::Error::Backend(
                    // XXX No thrilled about this allocation though
                    Error::NoXDisplayForName { 
                        name: x_display_name.map(|s| CString::new(
                            s.to_bytes_with_nul().to_owned()
                        ).unwrap_or_default())
                    }
                ));
            }
            let atoms = UsefulAtoms::fetch(x_dpy);
            let screen = x::XDefaultScreenOfDisplay(x_dpy);
            let screen_num = x::XDefaultScreen(x_dpy);
            let root = x::XRootWindowOfScreen(screen);
            let glx = Self::query_glx(x_dpy, screen_num);
            Ok(Self { x_dpy, atoms, screen, screen_num, root, glx })
        }
    }

    fn query_glx(x_dpy: *mut x::Display, screen_num: c_int) -> Option<Glx> {
        unsafe {
            // ??? What to do with error_base and event_base ?
            let (mut error_base, mut event_base) = mem::uninitialized();
            let has_glx = glx::glXQueryExtension(x_dpy, &mut error_base, &mut event_base);
            if has_glx == x::False {
                return None;
            }
        }

        let (major, minor) = unsafe {
            let (mut major, mut minor) = mem::uninitialized();
            let success = glx::glXQueryVersion(x_dpy, &mut major, &mut minor);
            if success == x::False {
               return None;
            }
            (major as u32, minor as u32)
        };
        let version = Semver::new(major, minor, 0);

        let get_proc_address = glx::glXGetProcAddress;

        if version < Semver::new(1,1,0) {
            // TODO Log this, it's terrible! A genuine dinosaur.
            return Some(Glx { version, get_proc_address,  ext: Default::default() });
        }

        let ext = unsafe {
            let client_vendor  = glx::glXGetClientString(  x_dpy, xx::GLX_VENDOR);
            let client_version = glx::glXGetClientString(  x_dpy, xx::GLX_VERSION);
            let server_vendor  = glx::glXQueryServerString(x_dpy, screen_num, xx::GLX_VENDOR);
            let server_version = glx::glXQueryServerString(x_dpy, screen_num, xx::GLX_VERSION);
            let extensions = glx::glXQueryExtensionsString(x_dpy, screen_num);
            GlxExt::parse(get_proc_address, &CStr::from_ptr(extensions))
        };
        Some(Glx { version, get_proc_address, ext })
    }
}









// NOTE: Not pub(super) because we want it to be accessible to the outside 
// world as handles.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Window<'dpy> {
    pub dpy: &'dpy Display,
    pub x_window: x::Window,
}

impl Display {
    pub(super) fn create_window<'dpy>(&'dpy self, settings: &Settings) 
        -> Result<Window<'dpy>, super::Error>
    {
        unsafe {
            let border_thickness = 0;
            let border_color = 0xff0000;
            let bg_color = 0x00ff00;
            let x_dpy = self.x_dpy;
            let parent = x::XDefaultRootWindow(x_dpy);

            let &Settings {
                mode, resizable, fully_opaque, opengl, allow_high_dpi
            } = settings;

            use super::window::Mode;
            let (w, h) = match mode {
                Mode::FixedSize(Extent2 { w, h }) => (w, h),
                // Mode::FixedSizeFullScreen => (),
                // Mode::DesktopSize =>,
                // Mode::FullScreen =>,
                _ => unimplemented!()
            };

            let x_window = x::XCreateSimpleWindow(x_dpy, parent,
                0, 0, w, h, border_thickness, border_color, bg_color);

            if x_window == 0 {
                return Err(super::Error::CouldntCreateWindow);
            }

            x::XFlush(x_dpy);

            Ok(Window { dpy: self, x_window })
        }
    }
}

impl<'dpy> Drop for Window<'dpy> {
    fn drop(&mut self) {
        unsafe {
            x::XDestroyWindow(self.dpy.x_dpy, self.x_window);
        }
    }
}

impl<'dpy> Window<'dpy> {


    pub(super) fn get_capabilities(&self) -> Capabilities {
        Capabilities {
            hide: WindowOpResult::Unimplemented,
            show: WindowOpResult::Success,
            set_title: WindowOpResult::Success,
            set_icon: WindowOpResult::Unimplemented,
            set_style: WindowOpResult::Unimplemented,
            recenter: WindowOpResult::Unimplemented,
            set_opacity: WindowOpResult::Unimplemented,
            maximize: WindowOpResult::Unimplemented,
            minimize: WindowOpResult::Unimplemented,
            restore: WindowOpResult::Unimplemented,
            raise: WindowOpResult::Unimplemented,
            enter_fullscreen: WindowOpResult::Unimplemented,
            leave_fullscreen: WindowOpResult::Unimplemented,
            set_minimum_size: WindowOpResult::Unimplemented,
            set_maximum_size: WindowOpResult::Unimplemented,
            move_absolute: WindowOpResult::Unimplemented,
            move_relative_to_parent: WindowOpResult::Unimplemented,
            move_relative_to_self: WindowOpResult::Unimplemented,
            resize: WindowOpResult::Unimplemented,
        }
    }

    unsafe extern fn _is_map_notify_callback(_x_dpy: *mut x::Display, ev: *mut x::XEvent, win: x::XPointer) -> i32 {
        let ev = ev.as_ref().unwrap();
        let xmap = x::XMapEvent::from(ev);
        let win = win as x::Window;
        (ev.get_type() == x::MapNotify && xmap.window == win) as i32
    }

    pub(super) fn show(&self) -> WindowOpResult {
        unsafe {
            let x_dpy = self.dpy.x_dpy;
            let x_window = self.x_window;
            // if !self._is_mapped() {
                x::XMapRaised(x_dpy, x_window);
                /*
                 * This blocks
                let mut event: x::XEvent = mem::uninitialized();
                x::XIfEvent(x_dpy, &mut event,
                    Some(Self::is_map_notify_callback),
                    x_window as x::XPointer);
                */
                x::XFlush(x_dpy);
            // }
            WindowOpResult::Success
        }
    }
    fn _is_mapped(&self) -> bool {
        unsafe {
            let mut attrs: x::XWindowAttributes = mem::uninitialized();
            x::XGetWindowAttributes(self.dpy.x_dpy, self.x_window, &mut attrs);
            attrs.map_state != x::IsUnmapped
        }
    }

    pub(super) fn create_child(&self, _settings: &Settings) -> Result<Self, super::Error> {
        unimplemented!()
    }
    pub(super) fn hide(&self) -> WindowOpResult { WindowOpResult::Unimplemented }
    pub(super) fn maximize(&self) -> WindowOpResult { WindowOpResult::Unimplemented }
    pub(super) fn minimize(&self) -> WindowOpResult { WindowOpResult::Unimplemented }
    pub(super) fn raise(&self) -> WindowOpResult { WindowOpResult::Unimplemented }
    pub(super) fn restore(&self) -> WindowOpResult { WindowOpResult::Unimplemented }
    pub(super) fn set_style(&self, _style: &Style) -> WindowOpResult {
        WindowOpResult::Unimplemented
    }
    pub(super) fn enter_fullscreen(&self) -> WindowOpResult { WindowOpResult::Unimplemented }
    pub(super) fn leave_fullscreen(&self) -> WindowOpResult { WindowOpResult::Unimplemented }
    pub(super) fn set_icon(&self, _icon: Option<Icon>) -> WindowOpResult {
        WindowOpResult::Unimplemented
    }
    pub(super) fn set_minimum_size(&self, _size: Extent2<u32>) -> WindowOpResult {
        WindowOpResult::Unimplemented
    }
    pub(super) fn set_maximum_size(&self, _size: Extent2<u32>) -> WindowOpResult {
        WindowOpResult::Unimplemented
    }
    pub(super) fn set_opacity(&self, _opacity: f32) -> WindowOpResult {
        WindowOpResult::Unimplemented
    }
    pub(super) fn move_absolute(&self, _pos: Extent2<u32>) -> WindowOpResult {
        WindowOpResult::Unimplemented
    }
    pub(super) fn move_relative_to_self(&self, _pos: Extent2<u32>) -> WindowOpResult {
        WindowOpResult::Unimplemented
    }
    pub(super) fn move_relative_to_parent(&self, _pos: Extent2<u32>) -> WindowOpResult {
        WindowOpResult::Unimplemented
    }
    pub(super) fn recenter(&self) -> WindowOpResult {
        WindowOpResult::Unimplemented
    }
    pub(super) fn resize(&self, _size: Extent2<u32>) -> WindowOpResult {
        WindowOpResult::Unimplemented
    }
    pub(super) fn set_title(&self, title: &str) -> WindowOpResult {
        unsafe {
            let mut title_prop: x::XTextProperty = mem::uninitialized();
            let title_ptr = CString::new(title).unwrap_or_default();
            let mut title_ptr = title_ptr.as_bytes_with_nul().as_ptr() as *mut u8;
            let title_ptr = &mut title_ptr as *mut _;
            let status = x::Xutf8TextListToTextProperty(
                self.dpy.x_dpy, mem::transmute(title_ptr), 1, x::XUTF8StringStyle, &mut title_prop
            );
            if status == x::Success as i32 {
                x::XSetTextProperty(self.dpy.x_dpy, self.x_window, &mut title_prop, self.dpy.atoms._NET_WM_NAME);
                x::XFree(title_prop.value as *mut _);
            }
            x::XFlush(self.dpy.x_dpy);
        }
        WindowOpResult::Success
    }
    pub(super) fn gl_swap_buffers(&self) {
        unsafe {
            glx::glXSwapBuffers(self.dpy.x_dpy, self.x_window)
        }
    }

    pub(super) fn query_screenspace_size(&self) -> Extent2<u32> {
        unimplemented!()
    }
    pub(super) fn query_canvas_size(&self) -> Extent2<u32> {
        unimplemented!()
    }

    pub(super) fn gl_set_swap_interval(&self, _interval: GLSwapInterval) -> Result<(),super::Error> { 
        unimplemented!()
    }
}











pub(super) struct GLContext<'dpy> {
    dpy: &'dpy Display,
    glx_context: glx::GLXContext,
}


impl Display {

    // gen_visual_attribs() and gen_fbconfig_attribs() are two separate 
    // functions for ease of maintenance. They don't have all keys in
    // common, and the format is different.
    // For instance, GLX_DOUBLEBUFFER and GLX_STEREO are not followed by
    // a boolean in gen_visual_attribs() - their presence _is_ the boolean
    // instead.

    // GLX below 1.3
    fn gen_visual_attribs(settings: &GLContextSettings) -> [c_int; 26] {
        let &GLContextSettings {
            depth_bits, stencil_bits, double_buffer, stereo,
            red_bits, blue_bits, green_bits, alpha_bits,
            accum_red_bits, accum_blue_bits, accum_green_bits, 
            accum_alpha_bits, aux_buffers, ..
        } = settings;
        let mut attr = [
            glx::GLX_RGBA,
            glx::GLX_AUX_BUFFERS, aux_buffers as c_int,
            glx::GLX_RED_SIZE, red_bits as c_int,
            glx::GLX_GREEN_SIZE, green_bits as c_int,
            glx::GLX_BLUE_SIZE, blue_bits as c_int,
            glx::GLX_ALPHA_SIZE, alpha_bits as c_int,
            glx::GLX_DEPTH_SIZE, depth_bits as c_int,
            glx::GLX_STENCIL_SIZE, stencil_bits as c_int,
            glx::GLX_ACCUM_RED_SIZE, accum_red_bits as c_int,
            glx::GLX_ACCUM_GREEN_SIZE, accum_green_bits as c_int,
            glx::GLX_ACCUM_BLUE_SIZE, accum_blue_bits as c_int,
            glx::GLX_ACCUM_ALPHA_SIZE, accum_alpha_bits as c_int,
            xx::None, // GLX_DOUBLEBUFFER
            xx::None, // GLX_STEREO
            xx::None // keep last
        ];
        let mut i = attr.len()-3;
        debug_assert_eq!(attr[i], xx::None);
        if double_buffer {
            attr[i] = glx::GLX_DOUBLEBUFFER;
            i += 1;
        }
        if stereo {
            attr[i] = glx::GLX_STEREO;
        }
        attr
    }

    // GLX 1.3 and above
    fn gen_fbconfig_attribs(settings: &GLContextSettings) -> [c_int; 37] {
        let &GLContextSettings {
            depth_bits, stencil_bits, double_buffer, stereo,
            red_bits, blue_bits, green_bits, alpha_bits,
            accum_red_bits, accum_blue_bits, accum_green_bits, 
            accum_alpha_bits, aux_buffers, ..
        } = settings;
        [
            glx::GLX_FBCONFIG_ID, glx::GLX_DONT_CARE,
            glx::GLX_DOUBLEBUFFER, double_buffer as c_int,
            glx::GLX_STEREO, stereo as c_int,
            glx::GLX_AUX_BUFFERS, aux_buffers as c_int,
            glx::GLX_RED_SIZE, red_bits as c_int,
            glx::GLX_GREEN_SIZE, green_bits as c_int,
            glx::GLX_BLUE_SIZE, blue_bits as c_int,
            glx::GLX_ALPHA_SIZE, alpha_bits as c_int,
            glx::GLX_DEPTH_SIZE, depth_bits as c_int,
            glx::GLX_STENCIL_SIZE, stencil_bits as c_int,
            glx::GLX_ACCUM_RED_SIZE, accum_red_bits as c_int,
            glx::GLX_ACCUM_GREEN_SIZE, accum_green_bits as c_int,
            glx::GLX_ACCUM_BLUE_SIZE, accum_blue_bits as c_int,
            glx::GLX_ACCUM_ALPHA_SIZE, accum_alpha_bits as c_int,
            glx::GLX_RENDER_TYPE, glx::GLX_RGBA_BIT,
            glx::GLX_DRAWABLE_TYPE, glx::GLX_WINDOW_BIT,
            glx::GLX_X_RENDERABLE, x::True,
            glx::GLX_TRANSPARENT_TYPE, xx::None, //glx::GLX_TRANSPARENT_RGB
            // glx::GLX_CONFIG_CAVEAT, xx::None,
            //
            // There's more GLX_TRANSPARENT_**_VALUE keys, might be
            // worth checking later,
            xx::None // keep last
        ]
    }


    pub(super) fn create_gl_context<'dpy>(
        &'dpy self, settings: &GLContextSettings
    ) -> Result<GLContext<'dpy>, super::Error>
    {
        if self.glx.is_none() {
            return Err(super::Error::Backend(Error::NoGLX));
        }

        let glx = self.glx.as_ref().unwrap();

        if glx.version < Semver::new(1,3,0) {
            // Not actually mutated, but glXChooseVisual wants *mut...
            let mut visual_attribs = Self::gen_visual_attribs(settings);
            let visual_info = unsafe { glx::glXChooseVisual(
                self.x_dpy, self.screen_num, visual_attribs.as_mut_ptr()
            )};
            if visual_info.is_null() {
                return Err(super::Error::Backend(Error::UnsupportedGLContextSettings));
            }

            unsafe {
                // TODO: we have the 'share' param here
                let glx_context = glx::glXCreateContext(
                    self.x_dpy, visual_info, ptr::null_mut(), x::True
                );
                x::XFree(visual_info as *mut _);
                return if glx_context.is_null() {
                    Err(super::Error::Backend(Error::FunctionName("glXCreateContext")))
                } else {
                    Ok(GLContext { dpy: self, glx_context })
                }
            }
        }

        // Here, we have GLX version >= 1.3

        let &GLContextSettings {
            version, debug, forward_compatible, profile, msaa, ..
        } = settings;


        // TODO ensure that the window has the appropriate pixel format
        // we should probably turn the opengl member of GLContextSettings
        // from bool to Option<GLContextSetting>... ?
        // Yes : we should move to XCreateWindow() and use the visual_info we got
        // from the glX functions.
        // We aren't able to recreate the window because it's probably shown by now.

        // See https://www.khronos.org/opengl/wiki/Tutorial:_OpenGL_3.0_Context_Creation_(GLX)

        let best_fbc: glx::GLXFBConfig = unimplemented!(); // FIXME
        let attribs_arb = [
            glx::arb::GLX_CONTEXT_MAJOR_VERSION_ARB, 3,
            glx::arb::GLX_CONTEXT_MINOR_VERSION_ARB, 0,
            glx::arb::GLX_CONTEXT_FLAGS_ARB, 
                glx::arb::GLX_CONTEXT_DEBUG_BIT_ARB |
                glx::arb::GLX_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB,
            glx::arb::GLX_CONTEXT_PROFILE_MASK_ARB,
            // Core is the default here
                glx::arb::GLX_CONTEXT_CORE_PROFILE_BIT_ARB |
                glx::arb::GLX_CONTEXT_COMPATIBILITY_PROFILE_BIT_ARB,
            xx::None
        ];

        // From the tutorial :
        // Install an X error handler so the application won't exit if GL 3.0
        // context allocation fails.
        // Note this error handler is global.  All display connections in all threads
        // of a process use the same error handler, so be sure to guard against other
        // threads issuing X commands while this code is running.

        /*
        ctxErrorOccurred = false;
        int (*oldHandler)(Display*, XErrorEvent*) = XSetErrorHandler(&ctxErrorHandler);
          
        if !hasExt(b"GLX_ARB_create_context\0") || dyn::glXCreateContextAttribsARB.is_none() {
            glXCreateNewContext(fbc);
        }

        else glXCreateContext(visual);
        */

        let glXCreateContextAttribsARB = glx.ext.glXCreateContextAttribsARB.unwrap();
        let glx_context = unsafe { glXCreateContextAttribsARB(
            self.x_dpy, best_fbc, ptr::null_mut(), x::True, attribs_arb.as_ptr()
        )};
        if glx_context.is_null() {
            return Err(super::Error::Backend(Error::FunctionName("GLXCreateContext")));
        }

        Ok(GLContext { dpy: self, glx_context })
    }



    pub(super) fn create_software_gl_context<'dpy>(&'dpy self, _settings: &GLContextSettings) -> Result<GLContext<'dpy>,super::Error> {
        unimplemented!()
    }
}


impl<'dpy> Drop for GLContext<'dpy> {
    fn drop(&mut self) {
        unsafe {
            // XXX Do we need to glXMakeCurrent() before destroying it ?
            glx::glXMakeCurrent(self.dpy.x_dpy, 0, ptr::null_mut());
            glx::glXDestroyContext(self.dpy.x_dpy, self.glx_context)
        }
    }
}

impl<'dpy> GLContext<'dpy> {
    pub(super) fn make_current(&self, win: &Window) {
        unsafe {
            glx::glXMakeCurrent(self.dpy.x_dpy, win.x_window, self.glx_context);
        }
    }

    // NOTE: glXGetProcAddressARB doesn't need a bound context, unlike
    // in WGL.
    pub(super) unsafe fn get_proc_address(&self, _name: *const c_char) -> Option<*const c_void> {
        unimplemented!()
    }
}

