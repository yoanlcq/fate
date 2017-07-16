//! The platform's display backend, windows, and OpenGL.
//!   
//! This module is not directly related to 2D/3D rendering (unlike the
//! Graphics module) - rather, it 
//! is here to provide handle(s) to the platform-specific display backend 
//! (analoguous to SDL2's video subsystem),
//! from which one can create potentially OpenGL-enabled windows.  
//!   
//! There are a few features that some targets (such as mobile OSes, browser 
//! environnments and consoles) might not support :  
//! 
//! - Multiple display contexts ([`Display`][1] objects);
//! - Multiple windows;
//! - Some operations on windows such as resizing.
//!   
//! When appropriate, the use of such features either reports errors you
//! can handle, or fails silently, depending on the case, but won't 
//! cause panics.
//!   
//! You are expected to at least "create" one window (which, on some 
//! targets, may actually just return the only window provided to
//! your app), and have an event loop.  
//! 
//! You might wonder what a [`Display`][1] is and why you would need one 
//! at all - the answer to this is : because it is the 
//! common denominator across all platforms, and it represents the minimum 
//! required
//! information/context shared between windows. Other libs which provide you 
//! with "out-of-the-box" windows are actually hiding this in the name 
//! of ease of use.  
//! On X11 for instance, a [`Display`][1] is a wrapper around an "X11 Display"
//! object (which represents a connection to an X server) and some more data
//! such as commonly-used X11 Atoms - but on other targets, it could very well
//! be an empty struct which does nothing.
//! 
//! A nice consequence is also that OpenGL contexts are not implictly tied
//! to a single window anymore. Rather, they are tied to a
//! [`Display`][1] context (which is more correct), which allows you to 
//! have a single OpenGL context shared across windows, even though this
//! might be discouraged.
//! 
//! 
//! # Examples
//! 
//! Creating a window:  
//! 
//! ```rust,no_run
//! use dmc::display::{window, Display};
//! # use dmc::display;
//! 
//! # fn foo() -> Result<(), display::Error> {
//! let display = Display::open()?;
//! let settings = window::Settings::from((800, 600));
//! # #[allow(unused_variables)]
//! let window = display.create_window(&settings)?;
//! # Ok(())
//! # }
//! # let _ = foo();
//! ```
//! 
//! Here, `settings` represents the absolute minimum information
//! that needs to be known at window creation time (here, we use its
//! From implementation which takes a fixed size and set all other fields
//! to their target-specific default values, but if you want, you can
//! initialize all of the fields one-by-one yourself).  
//! Anything else, such as the window's title, is optional and can be 
//! set later.
//! 
//! ```rust,no_run
//! # use dmc::display as dpy;
//! # fn foo() -> Result<(), dpy::Error> {
//! # let display = dpy::Display::open()?;
//! # let settings = dpy::window::Settings::default();
//! # let window = display.create_window(&settings)?;
//! window.set_title("Hello, display world!");
//! # Ok(())
//! # }
//! # let _ = foo();
//! ```
//! 
//! Then don't forget to show the window, because it makes sense for a 
//! window to be allocated but not "mapped" to the user's display.  
//! 
//! ```rust,no_run
//! # use dmc::display as dpy;
//! # fn foo() -> Result<(), dpy::Error> {
//! # let display =  dpy::Display::open()?;
//! # let settings = dpy::window::Settings::default();
//! # let window = display.create_window(&settings)?;
//! window.show();
//! # Ok(())
//! # }
//! # let _ = foo();
//! ```
//! 
//! Creating an OpenGL context:
//! 
//! ```rust,no_run
//! # use dmc::display;
//! # fn foo() -> Result<(), display::Error> {
//! # let display = display::Display::open()?;
//! // default() will pick the best for the target platform,
//! // but keep in mind that you can tweak it at will.
//! let settings = display::GLContextSettings::default();
//! # #[allow(unused_variables)]
//! let gl = display.create_gl_context(&settings)?;
//! # Ok(())
//! # }
//! # let _ = foo();
//! ```
//! 
//! Rendering with OpenGL:
//! 
//! ```rust,no_run
//! # use dmc::display;
//! # fn foo() -> Result<(), display::Error> {
//! # let display = display::Display::open()?;
//! # let gl = display.create_gl_context(&Default::default())?;
//! # let window = display.create_window(&Default::default())?;
//! let swap_chain = gl.make_current(&window);
//! 
//! use display::GLSwapInterval::*;
//! // GLSwapInterval offers more options, go check them out!
//! if swap_chain.set_interval(LateSwapTearing).is_err() {
//!     if swap_chain.set_interval(VSync).is_err() {
//!         let _ = swap_chain.set_interval(LimitFps(60));
//!     }
//! }
//! 
//! 'main_event_loop: loop {
//!     // Fetch and handle events...
//!     // glClear(...); ...
//!     // glDraw(...); ...
//!     swap_chain.present();
//!     # break;
//! }
//! # Ok(())
//! # }
//! # let _ = foo();
//! ```
//! 
//! [1]: struct.Display.html



#[cfg(dmc_display_backend="x11")]
#[path="x11.rs"]
mod backend;

// TODO
// - Enable/DisableScreenSaver
// - Get display modes
// - Get DPI
// - Get Grabbed window
// - More feature-complete messageboxes

use std::path::Path;
use std::os::raw::{c_void, c_char};
use Decision;
use Extent2;
use Rgba32;
use Semver;

pub mod window {

    //! Module related to window initialization and management.

    use super::Extent2;
    use super::Rgba32;
    use super::backend;
    use super::Error;
    use super::Decision;

    /// Full screen, or fixed-size.
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum Mode {
        #[allow(missing_docs)]
        FixedSize(Extent2<u32>),
        #[allow(missing_docs)]
        DesktopSize,
        #[allow(missing_docs)]
        FullScreen,
        /// This is _possible_ but I wonder who would use this.
        FixedSizeFullScreen(Extent2<u32>),
    }

    /// The absolute minimum information a window needs at creation time.
    /// 
    /// The `Default` implementation picks the most permissive values, except
    /// for `fully_opaque` which is set to `true`, because people seldom
    /// need semi-transparent windows.
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub struct Settings {
        /// Specifies whether you want a full-screen or fixed-size window.
        /// The default value is a `FixedSize` obtained by a heuristic
        /// based on the desktop's available size, which picks a size
        /// that leaves reasonable space around the window.
        pub mode: Mode,
        /// Support OpenGL contexts ? (defaults to `true`)
        pub opengl: bool,
        /// `true` by default -
        /// If `false`, the window won't be resizable, not even manually by
        /// the user. Also keep in mind that some targets (other than 
        /// desktop) don't support resizing at all, in which case this flag
        /// is silently ignored.
        pub resizable: bool,
        /// Some platforms (such as iOS and OS X) support high-dpi windows,
        /// which size in screen-coordinates then differ from their raster-
        /// coordinates size.
        /// 
        /// However this defaults to `false` because it might break some
        /// assumptions.
        pub allow_high_dpi: bool,
        /// Some windowing systems support semi-transparent windows, which
        /// is useful for making desktop companions, however it's better to
        /// let them know beforehand that you need such a feature.  
        /// This defaults to `true` because this is the most common.
        pub fully_opaque: bool,
    }

    impl Default for Mode {
        fn default() -> Self {
            // FIXME assuming a fixed window size
            Mode::FixedSize(Extent2::new(400, 300))
        }
    }

    impl Default for Settings {
        fn default() -> Self {
            Self {
                opengl: true,
                resizable: true,
                allow_high_dpi: true,
                fully_opaque: true,
                mode: Default::default(),
            }
        }
    }

    /// Actually a simple thickness-color pair.
    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub struct Borders {
        /// Thickness, in pixels. If `Auto`, use the window manager's default.
        pub thickness: Decision<u16>,
        /// If `Auto`, use the window manager's default.
        pub color: Decision<Rgba32>,
    }

    #[allow(missing_docs)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub struct TitleBarStyle {
        pub minimize_button: bool,
        pub maximize_button: bool,
        pub close_button: bool,
    }

    impl Default for TitleBarStyle {
        fn default() -> Self {
            Self {
                minimize_button: true,
                maximize_button: true,
                close_button: true,
            }
        }
    }

    /// Style hints for a window.
    pub struct Style {
        /// If `None`, the window won't have a title bar.
        pub title_bar: Option<TitleBarStyle>,
        /// If `None`, the window is borderless.
        pub borders: Option<Borders>,
    }

    /// TODO Data for a window's icon.
    pub struct Icon;


    impl<T: Into<Extent2<u32>>> From<T> for Mode {
        fn from(size: T) -> Self {
            Mode::FixedSize(size.into())
        }
    }

    impl<T: Into<Extent2<u32>>> From<T> for Settings {
        fn from(size: T) -> Self {
            Self {
                mode: Mode::from(size),
                .. Default::default()
            }
        }
    }

    /// Some operations for `Window`s might either fail, be unsupported or
    /// unimplemented.
    // NOTE: This used to be must_use, but it quickly started to look 
    // silly to having to call ignore() or "let _ = ..." on most results
    // from `Window`'s methods.
    // When people write `window.set_title("My Game");`, they don't expect
    // it to fail, and don't care that much if it's actually unsupported 
    // (e.g on consoles). In any case, they have probably checked out the
    // docs beforehand, in which case they know.
    //#[must_use = "Window operations often succeed, but might fail on some platforms, or if you did not opt-in for the relevant capabilities."]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum WindowOpResult {
        #[allow(missing_docs)]
        Success,
        #[allow(missing_docs)]
        Failed { reason: Option<&'static str> },
        /// An operation on a `Window` might be unsupported by the
        /// backend itself (not the implementation). In case the
        /// feature is actually supported AND the implementation is certain
        /// that you forgot to opt-in
        /// at window creation time (when filling the `Settings` struct),
        /// `required_optin` will be set to `true`.
        #[allow(missing_docs)]
        Unsupported { required_optin: bool },
        /// You shouldn't see this, because it's up to this crate to
        /// implement what it promises, but it may be more appropriate and
        /// better than panicking for the time being.
        Unimplemented,
    }

    impl WindowOpResult {
        /// Purposefully ignore this result.
        /// 
        /// This is equivalent to `let _ = window.some_method();`.
        pub fn ignore(self) {}
        /// Assert a value of `Success`, trigerring a panic if it is not.
        pub fn unwrap(self) {
            self.into_result().unwrap()
        }
        /// Consume this value by converting it to a standard `Result`.
        pub fn into_result(self) -> Result<(),Self> {
            match self {
                WindowOpResult::Success => Ok(()),
                _ => Err(self),
            }
        }
    }



    /// The set of capabilities a given window has.
    /// 
    /// Each member of this struct matches the relevant method's name
    /// for `Window` structs.
    #[allow(missing_docs)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub struct Capabilities {
        pub hide: WindowOpResult,
        pub show: WindowOpResult,
        pub set_title: WindowOpResult,
        pub set_icon: WindowOpResult,
        pub set_style: WindowOpResult,
        pub recenter: WindowOpResult,
        pub set_opacity: WindowOpResult,
        pub maximize: WindowOpResult,
        pub minimize: WindowOpResult,
        pub restore: WindowOpResult,
        pub raise: WindowOpResult,
        pub enter_fullscreen: WindowOpResult,
        pub leave_fullscreen: WindowOpResult,
        pub set_minimum_size: WindowOpResult,
        pub set_maximum_size: WindowOpResult,
        pub move_absolute: WindowOpResult,
        pub move_relative_to_parent: WindowOpResult,
        pub move_relative_to_self: WindowOpResult,
        pub resize: WindowOpResult,
    }

    /// The closest possible representation of a desktop window.
    /// 
    /// Such a concept might not exist on some platforms, in which case,
    /// the single window you "create" actually returns an abstraction
    /// over the "canvas" or "screen space" you're allowed to draw to.
    /// 
    /// See `Display` to see how to create them.
    #[derive(Debug)]
    pub struct Window<'dpy>(pub(super) backend::Window<'dpy>);

    impl<'a,'b:'a,'dpy:'b> Window<'dpy> {
        pub fn get_capabilities(&self) -> Capabilities {
            self.0.get_capabilities()
        }

        /// A window won't appear until this method is called.
        pub fn show(&self) -> WindowOpResult { self.0.show() }
        /// The obvious reciprocal of `show()`.
        pub fn hide(&self) -> WindowOpResult { self.0.hide() }

        /// Sets the window's title.
        pub fn set_title(&self, title: &str) -> WindowOpResult {
            self.0.set_title(title)
        }
        #[allow(missing_docs)]
        pub fn set_icon<I: Into<Option<Icon>>>(&self, icon: I) -> WindowOpResult {
            self.0.set_icon(icon.into())
        }
        /// Attempts to set the window's borders.
        pub fn set_style(&self, style: &Style) -> WindowOpResult {
            self.0.set_style(style)
        }
        /// Centers a window relatively to the space it is in, with regards to
        /// its size.
        pub fn recenter(&self) -> WindowOpResult {
            self.0.recenter()
        }

        /// Sets the window's opacity, provided the window was created with
        /// the `fully_opaque` flag set to `false`.
        /// 
        /// Valid values for `opacity` range from 0 to 1 (both inclusive).  
        /// You're expected to clamp the value yourself if needed.
        pub fn set_opacity(&self, opacity: f32) -> WindowOpResult {
            self.0.set_opacity(opacity)
        }


        /// Retrieves the window's internal implementation details, if you
        /// need to work around missing features.  
        /// 
        /// If that happens, you are welcome to report an issue!
        pub unsafe fn get_internal(&'b self) -> &'a backend::Window {
            &self.0
        }
        /// Retrieves the window's internal implementation details, if you
        /// need to work around missing features.  
        /// 
        /// If that happens, you are welcome to report an issue!
        pub unsafe fn get_internal_mut(&'dpy mut self) -> &'a mut backend::Window {
            &mut self.0
        }


        /// Attempts to create a child window.  
        /// 
        /// The definition of "child window" may vary slightly from platform
        /// to platform, but it's almost always just another window which is
        /// closed when its parent gets closed.
        pub fn create_child(&'b mut self, settings: &Settings) -> Result<Window<'a>,Error> { 
            self.0.create_child(settings).map(Window)
        }

        /// The window's size, in screen coordinates.
        /// 
        /// You should not rely on this being equal to its size
        /// in raster-space coordinates.  
        /// If you're interested in the "canvas"'s dimensions, 
        /// use the `query_canvas_size()` method instead.
        /// 
        /// The `query()` part means that the operation is possibly heavy and
        /// the result is not implicitly cached:  
        /// it's your responsibility to do so if this is what you want.
        pub fn query_screenspace_size(&self) -> Extent2<u32> {
            self.0.query_screenspace_size()
        }
        /// The window's size, in raster-space coordinates.  
        /// 
        /// On High-DPI-enabled windows, it should be bigger
        /// than the size in screen-coordinates.  
        /// This is what you should use for pixel-perfect rendering.
        /// 
        /// The `query()` part means that the operation is possibly heavy and
        /// the result is not implicitly cached:  
        /// it's your responsibility to do so if this is what you want.
        pub fn query_canvas_size(&self) -> Extent2<u32> {
            self.0.query_canvas_size()
        }


        /// Attempts to maximize the window (as in, take as much space as
        /// possible).
        pub fn maximize(&self) -> WindowOpResult { self.0.maximize() }
        /// Attempts to minimize the window (as in, minimize to task bar).
        pub fn minimize(&self) -> WindowOpResult { self.0.minimize() }
        /// The reciprocal of `minimize()`.
        pub fn restore(&self) -> WindowOpResult { self.0.restore() }
        /// Attempts to set the window on top of the stack and request focus.
        pub fn raise(&self) -> WindowOpResult { self.0.raise() }
        /// Attempts to go full-screen.
        /// 
        /// The `Window` struct doesn't keep track of an `is_fullscreen`
        /// boolean: it is yours to manage if you need one. This method
        /// won't perform the checks for you.
        /// However, for convenience, it saves the window's current size
        /// to automatically restore it whenever leaving full-screen mode.
        pub fn enter_fullscreen(&self) -> WindowOpResult { self.0.enter_fullscreen() }
        /// Attempts to leave full-screen mode.
        /// 
        /// See `enter_fullscreen()`.
        pub fn leave_fullscreen(&self) -> WindowOpResult { self.0.leave_fullscreen() }

        /// Unconditionnally prevents the window's size from going below the
        /// given threshold.
        pub fn set_minimum_size(&self, size: Extent2<u32>) -> WindowOpResult {
            self.0.set_minimum_size(size)
        }
        /// Unconditionnally prevents the window's size from going above the
        /// given threshold.
        pub fn set_maximum_size(&self, size: Extent2<u32>) -> WindowOpResult {
            self.0.set_maximum_size(size)
        }
        /// Moves the window to the given absolute position in 
        /// desktop-space.  
        /// 
        /// The anchor is the window's top-left corner.
        pub fn move_absolute(&self, pos: Extent2<u32>) -> WindowOpResult {
            self.0.move_absolute(pos)
        }
        /// Moves the window relatively to itself in 
        /// desktop-space.  
        /// 
        /// The anchor is the window's top-left corner.
        pub fn move_relative_to_self(&self, pos: Extent2<u32>) -> WindowOpResult {
            self.0.move_relative_to_self(pos)
        }
        /// Moves the window relatively to its parent, if any.  
        /// Otherwise, this is resolves to `move_absolute()`.
        /// 
        /// The anchor is the window's top-left corner.
        pub fn move_relative_to_parent(&self, pos: Extent2<u32>) -> WindowOpResult {
            self.0.move_relative_to_parent(pos)
        }
        /// Attempts to set the window's screen-space size.
        pub fn resize(&self, size: Extent2<u32>) -> WindowOpResult {
            self.0.resize(size)
        }
    }
}
use self::window::{Window, Settings, Style, Icon};

/// Error types returned by this module.
#[derive(Debug, Clone)]
pub enum Error {
    #[allow(missing_docs)]
    DoesntSupportMultipleWindows,
    #[allow(missing_docs)]
    CouldntCreateWindow,
    /// Backend-specific error.
    Backend(backend::Error)
}

/// A handle to the platform-specific display backend.
/// 
/// On X11, for instance, it wraps a connection to the X server.
#[derive(Debug)]
pub struct Display(backend::Display);

impl<'dpy> Display {
    /// Attempts to get one handle to the platform-specific display backend.
    /// 
    /// You should need only one.
    pub fn open() -> Result<Self, Error> {
        backend::Display::open().map(Display)
    }

    /// X11-only specialization of `open()` where you can specify
    /// the name given to `XOpenDisplay()`.
    #[cfg(dmc_display_backend="x11")]
    pub fn open_x11_display_name(name: Option<&::std::ffi::CStr>) -> Result<Self, Error> {
        backend::Display::open_x11_display_name(name).map(Display)
    }
    

    /// Attempts to create a `Window` with the given settings.
    pub fn create_window(&'dpy self, settings: &Settings) -> Result<Window<'dpy>, Error> {
        self.0.create_window(settings).map(Window)
    }
    /// Same as `create_window()`, but immediately shows the window afterwards
    /// if it succeeds.
    pub fn create_window_and_show(&'dpy self, settings: &Settings) -> Result<Window<'dpy>, Error> {
        let w = self.create_window(settings)?;
        w.show().ignore();
        Ok(w)
    }
    /// Attempts to create a backend-specific OpenGL context.
    pub fn create_gl_context(&'dpy self, settings: &GLContextSettings) -> Result<GLContext<'dpy>,Error> {
        self.0.create_gl_context(settings).map(GLContext)
    }
    /// Sames as `create_gl_context()`, but attempts to get a
    /// context that is not hardware-accelerated (on some platforms, this
    /// might try to load the Mesa driver).
    /// The use case for this is simple apps that don't specifically need a
    /// lot of perf, and would rather prefer saving battery power.
    /// 
    /// This won't attempt to fall back to the default implementation - in
    /// other words, this will succeed only if it is certain that there
    /// is a software implementation available AND a context could be created
    /// out of it.
    pub fn create_software_gl_context(&'dpy self, settings: &GLContextSettings) -> Result<GLContext<'dpy>,Error> {
        self.0.create_software_gl_context(settings).map(GLContext)
    }

    /// Attempts to create an OpenGL context from a dynamically-loaded 
    /// library.
    pub fn create_gl_context_from_lib<P: AsRef<Path>>(&'dpy self, _settings: &GLContextSettings, _path: P) -> Result<GLContext<'dpy>,Error> {
        unimplemented!()
    }
}


/// Wrapper around a platform-specific OpenGL Context.
pub struct GLContext<'dpy>(backend::GLContext<'dpy>);

/// Since OpenGL 3.2, the profile for an OpenGL context is either "core" 
/// or "compatibility".  
/// 
/// See [the relevant entry of the OpenGL wiki](https://www.khronos.org/opengl/wiki/Core_And_Compatibility_in_Contexts)
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum GLProfile {
    Core,
    Compatibility,
}

impl Default for GLProfile {
    fn default() -> Self {
        GLProfile::Compatibility
    }
}

/// Handle to a window's ability to swap OpenGL buffers.
/// 
/// The only way to get one is to call the `make_current` method
/// of a [`GLContext`](struct.GLContext.html).
pub struct GLSwapChain<'win,'gl:'win,'dpy:'gl> {
    window: &'win Window<'dpy>,
    gl_context: &'gl GLContext<'dpy>,
}


/// Either Desktop GL or OpenGL ES.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum GLVariant {
    Desktop,
    ES,
}

/// Known OpenGL version numbers.
/// 
/// If you're looking for WebGL, know that WebGL 
/// 1.0 maps closely to ES 2.0, and WebGL 2.0 maps closely to ES 3.0.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types, missing_docs)]
#[repr(u16)]
pub enum GLVersion {
    GL(Semver),
    ES(Semver),

    GL_4_5,
    GL_4_4,
    GL_4_3,
    GL_4_2,
    GL_4_1,
    GL_4_0,
    GL_3_3,
    GL_3_2,
    GL_3_1,
    GL_3_0,
    GL_2_1,
    GL_2_0,
    GL_1_5,
    GL_1_4,
    GL_1_3,
    GL_1_2_1,
    GL_1_2,
    GL_1_1,

    ES_3_2,
    ES_3_1,
    ES_3_0,
    ES_2_0,
    ES_1_1,
    ES_1_0,
}

impl GLVersion {
    #[allow(missing_docs)]
    // If None is returned, the user can still build a manual version
    // with the GL() and ES() variants.
    pub fn try_from_semver(v: &(GLVariant, Semver)) -> Option<Self> {
        let &(variant, Semver {major, minor, patch}) = v;
        match (variant, major, minor, patch) {
           (GLVariant::Desktop, 4,5,0) => Some(GLVersion::GL_4_5  ),
           (GLVariant::Desktop, 4,4,0) => Some(GLVersion::GL_4_4  ),
           (GLVariant::Desktop, 4,3,0) => Some(GLVersion::GL_4_3  ),
           (GLVariant::Desktop, 4,2,0) => Some(GLVersion::GL_4_2  ),
           (GLVariant::Desktop, 4,1,0) => Some(GLVersion::GL_4_1  ),
           (GLVariant::Desktop, 4,0,0) => Some(GLVersion::GL_4_0  ),
           (GLVariant::Desktop, 3,3,0) => Some(GLVersion::GL_3_3  ),
           (GLVariant::Desktop, 3,2,0) => Some(GLVersion::GL_3_2  ),
           (GLVariant::Desktop, 3,1,0) => Some(GLVersion::GL_3_1  ),
           (GLVariant::Desktop, 3,0,0) => Some(GLVersion::GL_3_0  ),
           (GLVariant::Desktop, 2,1,0) => Some(GLVersion::GL_2_1  ),
           (GLVariant::Desktop, 2,0,0) => Some(GLVersion::GL_2_0  ),
           (GLVariant::Desktop, 1,5,0) => Some(GLVersion::GL_1_5  ),
           (GLVariant::Desktop, 1,4,0) => Some(GLVersion::GL_1_4  ),
           (GLVariant::Desktop, 1,3,0) => Some(GLVersion::GL_1_3  ),
           (GLVariant::Desktop, 1,2,1) => Some(GLVersion::GL_1_2_1),
           (GLVariant::Desktop, 1,2,0) => Some(GLVersion::GL_1_2  ),
           (GLVariant::Desktop, 1,1,0) => Some(GLVersion::GL_1_1  ),
           (GLVariant::ES     , 3,2,0) => Some(GLVersion::ES_3_2  ),
           (GLVariant::ES     , 3,1,0) => Some(GLVersion::ES_3_1  ),
           (GLVariant::ES     , 3,0,0) => Some(GLVersion::ES_3_0  ),
           (GLVariant::ES     , 2,0,0) => Some(GLVersion::ES_2_0  ),
           (GLVariant::ES     , 1,1,0) => Some(GLVersion::ES_1_1  ),
           (GLVariant::ES     , 1,0,0) => Some(GLVersion::ES_1_0  ),
           _ => None,
        }
    }
    #[allow(missing_docs)]
    pub fn to_semver(&self) -> (GLVariant, Semver) {
        match *self {
            GLVersion::GL(v)    => (GLVariant::Desktop, v),
            GLVersion::ES(v)    => (GLVariant::ES     , v),
            GLVersion::GL_4_5   => (GLVariant::Desktop, Semver::new(4,5,0)),
            GLVersion::GL_4_4   => (GLVariant::Desktop, Semver::new(4,4,0)),
            GLVersion::GL_4_3   => (GLVariant::Desktop, Semver::new(4,3,0)),
            GLVersion::GL_4_2   => (GLVariant::Desktop, Semver::new(4,2,0)),
            GLVersion::GL_4_1   => (GLVariant::Desktop, Semver::new(4,1,0)),
            GLVersion::GL_4_0   => (GLVariant::Desktop, Semver::new(4,0,0)),
            GLVersion::GL_3_3   => (GLVariant::Desktop, Semver::new(3,3,0)),
            GLVersion::GL_3_2   => (GLVariant::Desktop, Semver::new(3,2,0)),
            GLVersion::GL_3_1   => (GLVariant::Desktop, Semver::new(3,1,0)),
            GLVersion::GL_3_0   => (GLVariant::Desktop, Semver::new(3,0,0)),
            GLVersion::GL_2_1   => (GLVariant::Desktop, Semver::new(2,1,0)),
            GLVersion::GL_2_0   => (GLVariant::Desktop, Semver::new(2,0,0)),
            GLVersion::GL_1_5   => (GLVariant::Desktop, Semver::new(1,5,0)),
            GLVersion::GL_1_4   => (GLVariant::Desktop, Semver::new(1,4,0)),
            GLVersion::GL_1_3   => (GLVariant::Desktop, Semver::new(1,3,0)),
            GLVersion::GL_1_2_1 => (GLVariant::Desktop, Semver::new(1,2,1)),
            GLVersion::GL_1_2   => (GLVariant::Desktop, Semver::new(1,2,0)),
            GLVersion::GL_1_1   => (GLVariant::Desktop, Semver::new(1,1,0)),
            GLVersion::ES_3_2   => (GLVariant::ES     , Semver::new(3,2,0)),
            GLVersion::ES_3_1   => (GLVariant::ES     , Semver::new(3,1,0)),
            GLVersion::ES_3_0   => (GLVariant::ES     , Semver::new(3,0,0)),
            GLVersion::ES_2_0   => (GLVariant::ES     , Semver::new(2,0,0)),
            GLVersion::ES_1_1   => (GLVariant::ES     , Semver::new(1,1,0)),
            GLVersion::ES_1_0   => (GLVariant::ES     , Semver::new(1,0,0)),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Requirement<'a, T: 'a> {
    /// The default.
    HighestAvailable,
    LowestAvailable,
    AtLeast(T),
    AtMost(T),
    Exactly(T),
    TryInOrder(&'a [T]),
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct GLMsaa {
    /// Number of MSAA buffers Should be 1.
    buffer_count: u8,
    /// Nmber of samples per pixel. Should be a power of two.
    sample_count: u8,
}

/// Settings requested for an OpenGL context.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct GLContextSettings<'a> {
    /// MultiSample AntiAliasing setting.
    pub msaa: Requirement<'a, GLMsaa>,
    /// Strategy to adopt regarding the OpenGL version when attempting
    /// to crate the OpenGL context.  
    /// The implementation will attempt to honor this in a best-effort
    /// basis, but cannot guarantee anything (some backends simply don't
    /// provide that level of control).
    /// This does not change the requested OpenGL profile.
    pub version: Requirement<'a, GLVersion>,
    /// Requirements (or not) a debug context.
    pub debug : bool,
    /// Only used when the requested OpenGL version is 3.0 or 
    /// greater.
    pub forward_compatible: bool, // 3.0+
    /// Only used vhen the requested OpenGL version is 3.2 or
    /// greater.
    /// 
    /// If you set it to Auto, the implementation will
    /// attempt to open a Compatibility profile, and if
    /// it fails, open a Core profile.
    pub profile : Decision<GLProfile>,
    /// Number of bits used for storing per-fragment depth values.  
    /// Often set to 24.
    pub depth_bits: u8,
    /// Number of bits used for storing per-fragment "stencil" values.
    /// Often set to 32-`depth_bits`.
    pub stencil_bits: u8,
    /// Use double-buffering ? Defaults to `true` because 
    /// not enabling this has been deprecated long ago.
    pub double_buffer: bool,
    /// Requirements "left" and "right" frame buffers instead of a single
    /// frame buffer (which is the default).
    /// Each said "frame buffer" can itself be double-buffered if 
    /// `double_buffer` was set to `true`.
    pub stereo: bool,
    /// Number of bits used for storing the red channel. Often set to 8.
    pub red_bits: u8,
    /// Number of bits used for storing the green channel. Often set to 8.
    pub green_bits: u8,
    /// Number of bits used for storing the blue channel. Often set to 8.
    pub blue_bits: u8,
    /// Number of bits used for storing the alpha channel. Often set to 8.
    pub alpha_bits: u8,
    /// Number of bits used for storing the red channel in the accumulation buffer, if any.
    pub accum_red_bits: u8,
    /// Number of bits used for storing the green channel in the accumulation buffer, if any.
    pub accum_green_bits: u8,
    /// Number of bits used for storing the blue channel in the accumulation buffer, if any.
    pub accum_blue_bits: u8,
    /// Number of bits used for storing the alpha channel in the accumulation buffer, if any.
    pub accum_alpha_bits: u8,
    /// Number of auxiliary image buffers.  
    /// This was deprecated since OpenGL 3.0.
    /// 
    /// See [The relevant section on the OpenGL
    /// wiki](https://www.khronos.org/opengl/wiki/Default_Framebuffer#Removed_buffer_images).
    pub aux_buffers: u8,
}


impl<'a> Default for GLContextSettings<'a> {
    fn default() -> Self {
        Self {
            msaa: Requirement::HighestAvailable,
            version: Requirement::HighestAvailable,
            .. Default::default()
        }
    }
}

impl<'a> GLContextSettings<'a> {
    /// TODO this function checks the correctness of these settings.
    /// For instance, it reports that not using double buffering is 
    /// deprecated.
    pub fn sanitize(&self) -> GLContextSettings<'static> {
        unimplemented!()
    }
}

impl<'win,'gl:'win,'dpy:'gl> GLContext<'dpy> {
    /// Lowers to the plaftorm-specific "<xxglxx>ContextMakeCurrent()",
    /// and handles back a "Swap Chain" object which lives as long as both
    /// the target window and the OpenGL context.
    pub fn make_current(&'gl self, window: &'win Window<'dpy>) -> GLSwapChain<'win,'gl,'dpy> {
        self.0.make_current(&window.0);
        let out = GLSwapChain { window, gl_context: self };
        if out.set_interval(Default::default()).is_err() {
            out.set_interval(GLSwapInterval::LimitFps(60)).unwrap();
        }
        out
    }

    /// Retrieves the OpenGL function pointer for the given name.
    pub unsafe fn get_proc_address(&self, name: *const c_char) -> Option<*const c_void> {
        self.0.get_proc_address(name)
    }
}

impl<'win,'gl:'win,'dpy:'gl> GLSwapChain<'win, 'gl, 'dpy> {
    /// Lowers to the plaftorm-specific `XXglXXSwapBuffers()`.
    /// Use this when you're done rendering the current frame.
    /// 
    /// Quoting SDL2's docs:  
    /// On Mac OS X make sure you bind 0 to the draw framebuffer before 
    /// swapping the window,
    /// otherwise nothing will happen. See [this blog
    /// post](http://renderingpipeline.com/2012/05/nsopenglcontext-flushbuffer-might-not-do-what-you-think/) for more info.
    pub fn present(&self) {
        self.window.0.gl_swap_buffers()
    }

    /// Attempts to set the chain's swap interval. 
    // FIXME: This doesn't need a GLSwapInterval enum
    pub fn set_interval(&self, interval: GLSwapInterval) -> Result<(),Error> {
        self.window.0.gl_set_swap_interval(interval)
    }
    /// You never need to do this unless you have several windows
    /// which share the same OpenGL Context.
    /// 
    /// Lowers to the plaftorm-specific `XXglXXContextMakeCurrent()`.
    pub fn force_make_current(&self) {
        self.gl_context.0.make_current(&self.window.0);
    }
}

/// The interval at which OpenGL buffers are swapped.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum GLSwapInterval {
    /// Vertical sync : frames are synchronized with the monitor's refresh 
    /// rate. This is the default.
    VSync,
    /// Immediate frame updates. May make your fan melt if you don't limit
    /// the FPS.
    Immediate,
    /// Prevents frames from being presented faster than the given
    /// frames-per-second limit.
    LimitFps(u32),
    /// Passed directly as the value for the backend's GL SwapInterval()
    /// function.  
    /// 
    /// - 0: Immediate updates,  
    /// - 1: Vsync  
    /// - 2: Vsync/2 (e.g at 60 Hz, will swap buffers every 30 frames.)  
    /// - etc...  
    Interval(u32),
    /// Quoting SDL2's docs:  
    /// Late swap tearing works the same as vsync, but if you've 
    /// already missed the vertical
    /// retrace for a given frame, it swaps buffers immediately, which
    /// might be less jarring for
    /// the user during occasional framerate drops.  
    ///   
    /// Late swap tearing is implemented for some glX drivers with
    /// GLX_EXT_swap_control_tear and for some Windows drivers with
    /// WGL_EXT_swap_control_tear.
    LateSwapTearing,
}

impl Default for GLSwapInterval {
    fn default() -> Self {
        GLSwapInterval::VSync
    }
}

/*

pub struct MessageBox;
pub struct MessageBoxBuilder;

impl MessageBox {
    pub fn new(_title: &str, _message: &str) -> MessageBoxBuilder {
        unimplemented!()
    }
}
impl MessageBoxBuilder {
    pub fn parent_window<'dpy, W: Into<Option<Window<'dpy>>>>(&mut self, _window: W) { unimplemented!() }
    pub fn error(self) -> Result<(),()> { unimplemented!() }
    pub fn warning(self) -> Result<(),()> { unimplemented!() }
    pub fn info(self) -> Result<(),()> { unimplemented!() }
}

pub struct Renderer2DConfig<'a,'dpy:'a> {
    pub window: &'a Window<'dpy>,
    pub software: bool,
    pub backend_name: &'a str,
}
pub struct Renderer2D {}

impl Renderer2D {
    pub fn backends<'a>() -> Option<Vec<Renderer2DConfig<'a>>> { unimplemented!() }
    pub fn new(_: &Renderer2DConfig) -> Self { unimplemented!() }
}

*/
