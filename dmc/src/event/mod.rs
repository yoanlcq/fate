//! Platform-specific event handling.

// TODO:
// - GetNumTouchDevices, etc
// - GetNumTouchFingers, etc
// - LoadDollarTemplates, SaveDollarTemplate, etc
// - RecordGesture
// - Start/StopTextInput

use std::time::Duration;
// XXX
use Extent2;
use Vec2;
use Timeout;


#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct VKey;
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Key;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Click {
    Single,
    Double,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Extra1,
    Extra2,
    Extra3,
    Other(u32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    AudioOutputDeviceAdded,
    AudioOutputDeviceRemoved,
    AudioCaptureDeviceAdded,
    AudioCaptureDeviceRemoved,
    // A Joystick should count as a Controller
    // A Joystick hat should count as an axis
    ControllerAxisMotion { axis_id: u32, axis: Vec2<i32> },
    ControllerButtonPressed { controller_id: u32, button: u32 },
    ControllerButtonReleased { controller_id: u32, button: u32 },
    ControllerTrackballMotion { controller_id: u32, ball_index: u8, motion: Vec2<i32> },
    ControllerAdded { controller_id: u32, },
    ControllerRemoved { controller_id: u32, },
    ControllerRemapped { controller_id: u32, },
    DollarGesture { touch_device_id: u32, gesture_id: u32, finger_count: u8, error: f32, normalized_center: Vec2<f32> },
    DragAndDropBegin,
    DragAndDropCancel,
    DragAndDropFile { file_path: String, },
    DragAndDropText { text: String, },
    DragAndDropRawData { text: Vec<u8>, },
    FingerPressed { touch_id: u32, finger_id: u32, normalized_position: Vec2<f32>, pressure: f32 },
    FingerReleased { touch_id: u32, finger_id: u32, normalized_position: Vec2<f32>, pressure: f32 },
    FingerMotion { touch_id: u32, finger_id: u32, normalized_motion: Vec2<f32>, pressure: f32 },
    KeyPressed { window_id: Option<u32>, is_repeat: bool, vkey: VKey, key: Key, },
    KeyReleased { window_id: Option<u32>, is_repeat: bool, vkey: VKey, key: Key, },
    MouseButtonPressed { window_id: Option<u32>, mouse: u32, click: Click, button: MouseButton, },
    MouseButtonReleased { window_id: Option<u32>, mouse: u32, click: Click, button: MouseButton, },
    MouseMotion { window_id: Option<u32>, mouse: u32, new_position: Vec2<i32> },
    MouseScroll { window_id: Option<u32>, mouse: u32, scroll: Vec2<i32>, },
    MultiGesture { touch_id: u32, theta: f32, dist: f32, normalized_center: Vec2<f32>, finger_count: u8 },

    WindowShown { window_id: u32, },
    WindowHidden { window_id: u32, },
    WindowShouldRedrawItself { window_id: u32, },
    WindowMoved { window_id: u32, position: Extent2<u32>, },
    WindowResized { window_id: u32, size: Extent2<u32>, by_user: bool, },
    WindowMinimized { window_id: u32, },
    WindowMaximized { window_id: u32, },
    WindowRestored { window_id: u32, },
    WindowGainedMouseFocus { window_id: u32, },
    WindowLostMouseFocus { window_id: u32, },
    WindowGainedKeyboardFocus { window_id: u32, },
    WindowLostKeyboardFocus { window_id: u32, },
    WindowCloseRequested { window_id: u32, },

    Quit,
    AppTerminating,
    AppLowMemory,
    AppEnteringBackground,
    AppEnteredBackground,
    AppEnteringForeground,
    AppEnteredForeground,

    KeymapChanged,
    ClipboardChanged,
    RenderTargetReset,
    DisplayLost,

    /// The text input buffer was updated ! Use get_text_input_buffer().
    TextInput,
}

#[derive(Debug, Hash)]
pub struct Clipboard {
    raw_data: Vec<u8>,
}
// XXX It's a singleton, how to implement this ?
impl Clipboard {
    pub fn get_raw_buffer<'a>(&'a self) -> &'a [u8] {
        &self.raw_data
    }
    pub fn overwrite_with_utf8(&mut self, _s: &str) {
        unimplemented!()
    }
}
#[derive(Debug, Hash)]
pub struct TextInput {
    raw_data: Vec<u8>,
}

// XXX It's a singleton, how to implement this ?
impl TextInput {
    // XXX is is practical for the user?
    pub fn start(&mut self) -> TextInputRecording { unimplemented!() }
}

pub struct TextInputRecording<'a> {
    _text_input: &'a TextInput,
}
impl<'a> TextInputRecording<'a> {
    pub fn get_raw_buffer(&'a self) -> &'a [u8] {
        unimplemented!()
    }
}


pub mod queue {

    use super::*;

    #[derive(Debug)]
    pub struct EventQueue {}

    #[derive(Debug)]
    pub struct PeekIter<'a> {
        _queue: &'a EventQueue,
    }
    #[derive(Debug)]
    pub struct PollIter<'a> {
        _queue: &'a mut EventQueue,
    }
    #[derive(Debug)]
    pub struct WaitIter<'a> {
        _queue: &'a mut EventQueue,
        _timeout: Timeout,
    }
    impl<'a> Iterator for PollIter<'a> {
        type Item = Event;
        fn next(&mut self) -> Option<Self::Item> {
            unimplemented!()
        }
    }

    impl<'a> Iterator for WaitIter<'a> {
        type Item = Event;
        fn next(&mut self) -> Option<Self::Item> {
            unimplemented!()
        }
    }

    impl<'a> Iterator for PeekIter<'a> {
        type Item = &'a Event;
        fn next(&mut self) -> Option<Self::Item> {
            unimplemented!()
        }
    }

    impl<'a> EventQueue {
        pub fn push(&mut self, _event: Event) { unimplemented!() }
        pub fn poll(&'a mut self) -> PollIter<'a> { 
            PollIter { _queue: self }
        }
        pub fn wait<T: Into<Timeout>>(&'a mut self, timeout: T) -> WaitIter<'a> { 
            WaitIter { _queue: self, _timeout: timeout.into() }
        }
        pub fn peek(&'a self) -> PeekIter<'a> {
            PeekIter { _queue: self }
        }
    }
}

pub use self::queue::EventQueue;

