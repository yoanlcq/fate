// TODO: Make this compile, and use the vek crate.

extern crate vek;

use self::vek::{Xy, Extent2, Rect, Rgba32, Uv};

pub struct Context2D;

impl Context2D {
    pub fn input(&mut self) -> InputStep {
        unimplemented!()
    }
}

pub type RasterXy = Xy<i32>;
pub type RasterExtent2 = Extent2<u32>;
pub type RasterRect = Rect<i32, u32>;

pub enum KeyState {
    Pressed,
    Released,
}

pub struct MouseState {
    pub position: RasterXy,
    pub scroll: RasterXy,
    pub left: KeyState,
    pub middle: KeyState,
    pub right: KeyState,
}

pub struct InputStep {
    pub mouse: MouseState,
    pub tab: KeyState,
}

impl InputStep {
    pub fn string(&mut self, _s: &str) {
        unimplemented!()
    }
    pub fn end(self) -> (GuiBuilder, DrawCommandIterator) {
        unimplemented!()
    }
}

pub struct GuiBuilder {}

pub struct WindowChoice {}
pub struct FixedSizeWindowBuilder {}
pub struct FlexWindowBuilder {}

impl WindowChoice {
    pub fn fixed_size(self) -> FixedSizeWindowBuilder {
        unimplemented!()
    }
    pub fn flex(self) -> FlexWindowBuilder {
        unimplemented!()
    }
}

impl GuiBuilder {
    pub fn window<F: Fn()>(&mut self, _f: F) -> WindowChoice {
        unimplemented!()
    }
    pub fn menubar(&mut self) -> MenuBarBuilder {
        unimplemented!()
    }
    // NEED:
    // - Workspaces
    // - 2D vs 3D world
    // - Smooth Transitions/animations (Linear or not)
    // - Animated icons
    // - Notification toasts
    // - FriendPaint-style inset widgets ?
    // - Get inspired by Blender UI ?
    // - File Explorer ? Vi editor ?
    // - Clipboard ops ?
    // TODO Dialog box ?
    // TODO Tree view ?
}

pub struct MenuBarBuilder {}

impl MenuBarBuilder {
    pub fn dropdown  (&mut self) { unimplemented!() }
    pub fn button    (&mut self) { unimplemented!() }
    pub fn separator (&mut self) { unimplemented!() }
    pub fn tasks     (&mut self) { unimplemented!() } // Where all windows are minimized
    pub fn workspaces(&mut self) { unimplemented!() }
    pub fn label     (&mut self) { unimplemented!() }
    // TODO ribbon bar ?
}

pub struct Style {
    // TODO
    // - Color palette
    // - Cursor style
}

// TODO
// Images ? We don't even need to store their data, since they are
// user-backed, but in this case we at least need to identify them
// in draw commands.

pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
}
pub enum VerticalAlignment {
    Top,
    Middle,
    Bottom,
}

pub trait TextEffect {
    //...
}
pub struct BoldOn;
impl TextEffect for BoldOn {}
impl TextEffect for Rgba32 {}

//pub struct StyledString(&[(&TextEffect, &str)]);

pub struct Title<'a> {
    //pub text: StyledString,
    pub font: &'a Font<'a>,
    pub horizontal_alignment: HorizontalAlignment,
}

pub struct WindowCapabilities {
    pub movable: bool,
    pub scalable: bool,
    pub closeable: bool,
    pub minimizable: bool,
    pub no_scrollbar: bool,
    pub no_input: bool,
}
//impl Default for WindowCapabilities {}

pub struct Border {
    pub thickness: u32,
    pub color: Rgba32,
}

pub struct WindowStyle {
    pub border: Border,
}

pub struct Canvas {}
pub struct SliderOpts {}
pub struct WindowHandle {}

pub struct WindowBuilder<'a, 'canvas> {
    _title: Option<Title<'a>>,
    _capabilities: WindowCapabilities,
    _style: WindowStyle,
    _position: RasterXy,
    _size: RasterXy,
    _canvas: &'canvas mut Canvas,
}

pub struct WidgetData {}

impl WidgetData {
    pub fn get_rect() {}
    pub fn is_hovered() -> bool { unimplemented!() }
    pub fn was_pressed() -> bool { unimplemented!() }
    pub fn is_being_pressed() -> bool { unimplemented!() }
    // nk_spacing ?? reserve space ??
    // TODO
    // - Any widget has an associated contextual menu, editable by the user;
    // - Any widget has an associated tooltip;
}

// TODO: Provide access to the internal GUI state so that users can, for 
// instance, iterate over all windows to do something, etc.

impl<'a, 'canvas> WindowBuilder<'a, 'canvas> {
    pub fn has_focus() -> bool { unimplemented!() }
    pub fn focus() { unimplemented!() } // Focus this window
    pub fn is_collapsed() -> bool { unimplemented!() }
    pub fn collapse() { unimplemented!() }
    pub fn expand() { unimplemented!() }
    pub fn is_closed() -> bool { unimplemented!() }
    pub fn close() { unimplemented!() }
    pub fn is_hidden() -> bool { unimplemented!() }
    pub fn show() { unimplemented!() }
    pub fn hide() { unimplemented!() }
    pub fn is_hovered() -> bool { unimplemented!() }
    pub fn get_content_region() -> RasterRect { unimplemented!() }

    pub fn child_window() { unimplemented!() }
    pub fn popup() { unimplemented!() }
    pub fn alert() { unimplemented!() } // Can only be one alert() at a given time.

    // TODO Layouts
    // - insets (top, bottom, left, right)
    //   - fixed
    //   - flex (custom computation closures)
    // - rows
    //   - wrap, nowrap,
    //   - manual new-row-feed
    //   - each row can align its content differently: left, center, right
    // - grid layout
    //   - elements _may_ span on one or more cells;;
    //   - each row+col size can be either fixed or flex (custom computation 
    //     closures)
    // TODO margins and padding
    // TODO tabs ?
    
    // Widget containers
    /// A group is for a set of widgets which share the same scrollbars.
    pub fn group() { unimplemented!() }
    /// Same as group, but each element can be toggled.
    pub fn list_view() { unimplemented!() }
    /// Same as list_view, but it's a hierarchy.
    pub fn tree() { unimplemented!() }

    // Widgets
    pub fn dropdown_list() { unimplemented!() }
    pub fn text() { unimplemented!() }
    pub fn label() { /* TODO may be hyperlink ? */ }
    pub fn button() {
        // Either has a symbol or image, then a text or label.
        // It also has an action when it's clicked.
        // Also, the trigger that causes it to fire the action. (On click ? On release ?
        // On release, but not if not hovered ?)
        // Couldn't we just have complete control over its contents ?
    }
    pub fn checkbox() { unimplemented!() }
    pub fn radio() { /*these must be part of a group*/}
    pub fn horizontal_slider(&mut self, _s: SliderOpts) { unimplemented!() }
    pub fn vertical_slider(&mut self, _s: SliderOpts) { unimplemented!() }
    pub fn progressbar() { unimplemented!() }
    pub fn color_picker() { unimplemented!() }
    // There are no "properties" like in Nuklear. There are only text fields
    // and conversions back and forth from any type to a string.
    // A text field, however, may have filters (e.g only accept and display hex, or URLs, etc)
    pub fn text_field() { unimplemented!() }
    pub fn text_editor() { /* TODO details + undo/redo + clipboard ops */ }
    pub fn chart() { unimplemented!() }
    pub fn combo_box() { unimplemented!() }

    pub fn end() -> WindowHandle { unimplemented!() }
}

impl<'a, 'canvas> Drop for WindowBuilder<'a, 'canvas> {
    // The window is done, call end() ignoring the return value
    fn drop(&mut self) {
        unimplemented!()
    }
}

impl Drop for GuiBuilder {
    fn drop(&mut self) {
        // Release the lock for the draw command iterator
    }
}

pub struct DrawCommandIterator {}
//impl Iterator for DrawCommandIterator {}

// TODO All DrawCommand stuff
// + All functions which allow to push new commands into the queue



pub struct Font<'a> {
    _callback: &'a FontTrait,
    _height: f32,
}

pub struct GlyphInfo {
    /// size of the glyph
    _extent: Extent2<f32>,
    /// offset to the next glyph
    _xadvance: f32,
    /// texture coordinates
    _uv: Uv<f32>,
    /// offset between top left and glyph
    _offset: Xy<f32>,
}

pub trait FontTrait {
    fn get_text_width(&self, font_height: f32, _text: &str) -> f32;
    // Default impl
    fn get_glyph_info(&self, _font_height: f32, _codepoint: char, _next_codepoint: char) -> GlyphInfo {
        unimplemented!()
        /*
        GlyphInfo {
            extent: unimplemented!(), //TODO
            xadvance: unimplemented!(), //TODO
            uv: Default::default(),
            offset: unimplemented!(), // TODO
        }
        */
    }
}

// TODO
// Embed font baker ? TTF support ? Default font support ?


// TODO
// Memory buffer extension ? (with memory control ?)

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
