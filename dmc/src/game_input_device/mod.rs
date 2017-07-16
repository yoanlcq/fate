//! Get input from gamepads, joysticks, steering wheels, and others.
//! 
//! Within this module's doc, a "device" will refer to any input device
//! specific to video-games, such as gamepads, joysticks and steering wheels.
//! 
//! # Background
//! 
//! Cross-platform input with game input devices is a much more of a big
//! deal than it should be.
//! Especially on desktop, there's such a wide variety of devices, each 
//! with their own
//! button sets, capabilities, OS-and-driver-and-backend specific
//! quirks, that it's seldom possible to properly categorize one when the
//! OS exposes it.  
//! This is particularly the case on Linux, where the situation is
//! a complete mess to the point most users have to configure everything
//! from scratch (and let's not get started on BSD and friends).  
//! On the opposite side (such as Windows with XInput), things are 
//! much smoother.  
//! 
//! With this in mind, it's near impossible to provide wrappers that
//! cover accurately every possible device - most multimedia libraries 
//! (such as SDL2 and SFML) afford to lose some information in favor of a 
//! simpler interface, and some engines (eg. Unity) "stringly-type" parts
//! of their related APIs.  
//! 
//! This module strives to achieve professional-quality completeness,
//! and as such, attempts to provide every useful piece of information
//! the backend exposes and which could be of reasonable interest to
//! developers, players, and testers.
//! As such, it is not as small or simple, but it represents well the current
//! situation.
//! 
//! This is also why it is not named "gamepad",
//! "game_controller", "joystick", or anything else, because these names
//! carry too many assumptions.
//! 
//! # Overview
//! 
//! This module's main entry point is the `GameInputDevice` structure,
//! which is a handle to a connected device. You acquire these by
//! first creating a `Monitor` once. At this time, you also get a list
//! of the already connected devices, which you have to take ownership of
//! (not _necessarily_ the list itself, but rather, each device).
//! 
//! Then, you _should_ poll the `Monitor` regularly to know when 
//! new devices are connected.
//! 
//! Device handles which the `Monitor` gives are yours to 
//! manage (unlike some libraries which hand back IDs or indices to
//! internal collections). Again: **this module does not keep track of device 
//! handles. If you miss the opportunities to store them, you won't 
//! be able to get them back later.**
//! 
//! You should _also_ poll your own list of `GameInputDevice`s regularly
//! to receive state change events from each one of them.
//! One of these events is 
//! `Disconnected`, which tells you that the device isn't available anymore,
//! in which case you should drop the
//! `GameInputDevice` in order to free the remaining resources, and because
//! you won't be able to do much with it anymore.
//! 
//! Regardless of a device's actual classification and abilities, you should
//! handle all possible events, even when some of them may actually never
//! be generated. For instance, an actual gamepad will never report
//! joystick-specific events, and vice versa, but sometimes the backend
//! just doesn't know and reports whatever event it deems appropriate.
//! 
//! You can do a bunch of other things with a `GameInputDevice`, some of
//! which are:
//! 
//! - Query the immediate state of its buttons and axes;
//! - Query its battery's immediate state, if relevant and supported;
//! - Retrieve associated metadata, such as its name, vendor, capabilities,
//!   etc;
//! - Send a list of event remapping "passes" (if needed);
//! - Send and play a variety of haptic (force feedback) effects;
//! 
//! Some things that aren't supported yet, but are put in a potential TODO:
//! 
//! - Capture from integrated microphones;
//! - Output audio from integrated speakers;
//! 
//! 
//! # The model
//! 
//! TODO: Picture of the virtual gamepad model ?
//! 
//! In this module: 
//! 
//! - X axes go from left (negative) to right (positive);
//! - Y and Z axes go from down (negative) to up (positive);
//! - Integer axis values are normalized to the `[min, max]` inclusive 
//!   range, where `min` and `max` are the minimum and maximum values of the
//!   integer type.
//! 
//! This module also does *not* implicitly remap actions (except for 
//! cases when the backend is known not to be sane).  
//! Right now, it's up to your game to provide proper input remapping.
//! This module might be extended later to ease this task.
//! 
//! # Advice for input handling
//! 
//! ## Dead zones
//! Often, gamepad axes are very sensitive, such that what should be a 
//! complete absence of motion for the end user is actually reported as very 
//! tiny motion events. 
//! Since this causes poor gameplay experience, there's that concept 
//! of "dead zones" within which your game should ignore motion events.  
//! Some drivers claim to cull such events automatically (even though 
//! sometimes they actually don't), some have no idea about a device's 
//! appropriate dead zones (while some do, or think they do)...
//! 
//! Long story short, it's a complete mess.  
//! So, you should decide of your own dead zone values (or better, let the 
//! user configure them) and manually ignore events which do not comply.
//! When the driver exposes dead zone values, this module
//! provides them to you, but I don't know whether you should actually 
//! trust them or not - it's up to you and your game.  
//! 
//! Proper handling of dead zones is not self-evident: you're encouraged
//! to use the "Scaled Radial Dead Zone" technique as described in 
//! [_Doing Thumbstick Dead Zones Right_](http://www.third-helix.com/2013/04/12/doing-thumbstick-dead-zones-right.html).
//! 
//! ## `dpad` vs. `hat[0]`
//! 
//! On some (most ?) Linux drivers, D-pad motions are actually reported as
//! `joystick_hats[0]`. It is probably wise to handle `dpad` and
//! `joystick_hats[0]`
//! as being mutually exclusive *and* meaning the same thing.
//! 
//! ## `L2`/`R2` vs. `LTrigger`/`RTrigger`
//! 
//! What you know as the `L2` and `R2` buttons on gamepads are actually 
//! *triggers* on Xbox360 gamepads (as such, they are axes, not buttons).  
//! You should handle both `*_shoulder_2` and `*_trigger`
//! as being mutually exclusive *and* meaning the same thing, knowing that
//! `*_trigger` carries more precision.
//! 
//! ## Primary stick
//! Joystick devices report joystick motion through `l_stick` (which in
//! this case means "the first stick" instead of actual "left stick").
//! 
//! ## `l_stick.z`/`r_stick.z` vs `l_thumb`/`r_thumb`
//! `*_stick.z` is for joysticks which genuinely support vertical motion as
//! an axis.  
//! `*_thumb` is for gamepad joysticks which can be "clicked" (as for
//! Dualshock and Xbox 360 gamepads, and most others actually).
//! 
//! ## `joystick_thumb1` vs `l_thumb`
//! You probably want to handle both as if they were the same, but it depends.
//! 
//! # Implementation notes
//! 
//! ## Linux
//! 
//! This module uses `libudev` to monitor and manage devices.
//! When a device is connected, it normally exposes both 
//! `/dev/input/jsXX` (`joydev` node) and `/dev/input/eventYY` (`evdev` node),
//! where XX and YY are integers not known in advance.  
//! Normally, user-space is granted read-write access to these nodes
//! (other devices, such as
//! mouses and keyboards, are provided to user-space by the X server).
//! 
//! It then attempts to open the `evdev` node. If it succeeds, further
//! interactions with the device are performed through `libevdev`.
//! 
//! Otherwise, it attempts to open the `joydev` node. If it succeeds, 
//! further interactions with the device are performed though standard
//! `read()` and `write()` calls, guided by struct definitions from
//! the `linux/joystick.h` system header file.
//! 
//! `joydev` is a bit too simple, in that the meaning of axes is fairly 
//! arbitrary, and buttons are only identified as a number from 0 to 10.  
//! `evdev` is the modern and more powerful alternative, which is why
//! it is preferred when available.

// TODO custom impl Debug for devices, which logs al udev info about it
// TODO haptic for mouses and keyboards


// NOTE: Cool doc https://wiki.archlinux.org/index.php/Gamepad

extern crate num_traits;
extern crate uuid;

use self::num_traits::{Signed, Num, ToPrimitive};
use self::uuid::Uuid as Guid;

use std;
use std::time::Instant;

use Semver;
use Knowledge;
use Timeout;
use BatteryState;


// XXX This was defined in FATE
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(missing_docs)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}
// XXX This was defined in FATE
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(missing_docs)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

/// A simple min-max value pair.
#[allow(missing_docs)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Minmax<T> {
    pub min: T,
    pub max: T,
}

/// A type that's suitable for representing a signed axis.
pub trait SignedAxis: Signed {
    /// (-1,1) for real numbers, (MIN,MAX) for integers.
    // FIXME: This wants to use associated constants
    fn normalized_axis_bounds() -> Minmax<Self>;
}

impl SignedAxis for f32 {
    fn normalized_axis_bounds() -> Minmax<Self> {
        Minmax {
            min: -1_f32,
            max: 1_f32,
        }
    }
}
impl SignedAxis for i8 {
    fn normalized_axis_bounds() -> Minmax<Self> { 
        Minmax {
            min: std::i8::MIN,
            max: std::i8::MAX,
        }
    }
}
impl SignedAxis for i16 {
    fn normalized_axis_bounds() -> Minmax<Self> {
        Minmax {
            min: std::i16::MIN,
            max: std::i16::MAX,
        }
    }
}
impl SignedAxis for i32 {
    fn normalized_axis_bounds() -> Minmax<Self> {
        Minmax {
            min: std::i32::MIN,
            max: std::i32::MAX,
        }
    }
}

/// A type that's suitable for representing an unsigned axis.
pub trait UnsignedAxis: Num /* Not Unsigned because we want floats */ {
    /// (-1,1) for real numbers, (MIN,MAX) for integers.
    // FIXME: This wants to use associated constants
    fn normalized_axis_bounds() -> Minmax<Self>;
}

impl UnsignedAxis for f32 {
    fn normalized_axis_bounds() -> Minmax<Self> {
        Minmax {
            min: 0_f32,
            max: 1_f32,
        }
    }
}
impl UnsignedAxis for u8 {
    fn normalized_axis_bounds() -> Minmax<Self> { 
        Minmax {
            min: 0,
            max: std::u8::MAX,
        }
    }
}
impl UnsignedAxis for u16 {
    fn normalized_axis_bounds() -> Minmax<Self> {
        Minmax {
            min: 0,
            max: std::u16::MAX,
        }
    }
}
impl UnsignedAxis for u32 {
    fn normalized_axis_bounds() -> Minmax<Self> {
        Minmax {
            min: 0,
            max: std::u32::MAX,
        }
    }
}

/// The well-known D-pad is a cross of 4 direction buttons.
#[allow(missing_docs)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Dpad {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Dpad {
    /// Get a (X,Y) absolute axis pair from this D-pad.
    /// 
    /// The values are as returned by `T::normalized_axis_bounds()`.
    pub fn to_axes<T: SignedAxis>(&self) -> Vec2<T> {
        let Minmax { min, max } = T::normalized_axis_bounds();
        let x = if self.down { min } else if self.up { max } else { T::zero() };
        let Minmax { min, max } = T::normalized_axis_bounds();
        let y = if self.left { min } else if self.right { max } else { T::zero() };
        Vec2 { x, y }
    }
}

/// The maximum number of hats for joystick devices.
pub const MAX_JOYSTICK_HATS: usize = 4;
/// The maximum number of base buttons (whatever that 
/// means) for joystick devices.
pub const MAX_JOYSTICK_BASE_BUTTONS: usize = 6;
/// The maximum number of numbered buttons.
/// 
/// NOTE: Linux evdev supports up to 10
pub const MAX_NUMBERED_BUTTONS: usize = 16;

/// Information for an axis, such as dead zone values, resolution and fuzz.
///
/// There's no 'minmax' field representing an axis' bounds: values are always
/// normalized for you.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AxisInfo<T> {
    /// Motion region within which you should not handle events from 
    /// this axis.  
    /// 
    /// Most devices won't expose this: you should thus use your own default
    /// dead zone value if there's none.
    /// 
    /// It's seldom incorrect to assume that min==max here.
    // NOTE: It's input_absinfo::flat in Linux and joydev claims to discard
    // motions automatically based on this
    // On the Windows-side, XInput has the INPUT_DEADZONE constant
    // but won't drop events based on this.
    pub dead_zone: Knowledge<Minmax<T>>,

    /// Quoting `linux/input.h`:  
    /// Resolution for main axes is reported in
    /// units per millimeter (units/mm), resolution for rotational axes
    /// is reported in units per radian.
    pub resolution: Knowledge<T>,

    /// Quoting `linux/input.h`:  
    /// Specifies fuzz value that is used to filter noise from
    /// the event stream.
    pub fuzz: Knowledge<T>,

    /// Actual number in bits the backend provides for this axis
    pub actual_value_type_bits: Knowledge<u8>,

    /// Whether this axis is actually provided by the backend as an integer.
    /// Otherwise, it's very probably a floating-point number.
    pub is_actual_integer: Knowledge<bool>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenericIAxis<T: SignedAxis>(pub T);
/// The common type for signed axes.
/// 
/// It is 32-bits because this is the precision offered by Linux's `evdev`.  
/// 
/// It is an integer because most backends actually provides these as
/// integers. Microsoft's `XInput`, and Linux's `joydev`, for instance, 
/// hand back signed 16-bit integers.
/// 
/// The values are normalized to `i32::MIN ... i32::MAX` for you.  
/// If you find working with normalized real numbers more convenient,
/// you can use the `to_normalized_f32()` method.
pub type IAxis = GenericIAxis<i32>;

impl<T: SignedAxis + ToPrimitive> GenericIAxis<T> {
    pub fn to_normalized_f32(&self) -> f32 {
        let Minmax { min, max } = T::normalized_axis_bounds();
        self.0.to_f32().unwrap() / if self.0.is_positive() { max } else { min }.to_f32().unwrap()
    }
    pub fn to_normalized_signed_axis<S: SignedAxis>(&self) -> T { 
        unimplemented!()
    }
    pub fn to_normalized_unsigned_axis<U: UnsignedAxis>(&self) -> T { 
        unimplemented!() 
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenericUAxis<T>(pub T);
/// The common type for unsigned axes.  
/// 
/// The values are normalized to `u32::MIN ... u32::MAX` for you.  
/// If you find working with normalized real numbers more convenient,
/// you can use the `to_normalized_f32()` method.
pub type UAxis = GenericUAxis<u32>;

impl UAxis {
    pub fn to_normalized_f32(&self) -> f32 { unimplemented!() }
    pub fn to_normalized_signed_axis<T: SignedAxis>(&self) -> T { 
        unimplemented!()
    }
    pub fn to_normalized_unsigned_axis<T: UnsignedAxis>(&self) -> T { 
        unimplemented!() 
    }
}

/// Linux evdev reports axis as 32-bit values
macro_rules! state {
    (st btn) => { bool };
    (st numbered_btns) => { [bool; MAX_NUMBERED_BUTTONS] };
    (st uaxis) => { UAxis };
    (st iaxis) => { IAxis };
    (st iaxis3) => { Vec3<IAxis> };
    (st hats) => { [Vec2<UAxis>; MAX_JOYSTICK_HATS] };
    (st Dpad) => { Dpad };

    (cap btn) => { Knowledge<Option<()>> };
    (cap numbered_btns) => { [Knowledge<Option<()>>; MAX_NUMBERED_BUTTONS] };
    (cap uaxis) =>  { Knowledge<Option<AxisInfo<UAxis>>> };
    (cap iaxis) =>  { Knowledge<Option<AxisInfo<IAxis>>> };
    (cap iaxis3) => { Knowledge<Vec3<Option<AxisInfo<IAxis>>>> };
    (cap hats) => { [ Knowledge<Option<AxisInfo<IAxis>>>; MAX_JOYSTICK_HATS] };
    (cap Dpad) => { Knowledge<Option<()>> };

    ($($(#[$attr:meta])* => $name:ident: $ty:ident),+) => {
        #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
        /// State of all of a device's buttons and axes.
        /// 
        /// `linux/input-event-codes.h` was a useful reference for this.
        pub struct State {
            $(
                $(#[$attr])*
                pub $name: state!(st $ty),
            )+
        }
        /// Which buttons and axes the device supports, as claimed by the
        /// backend.
        ///
        /// Each member is a `Knowledge<Option<T>>`. Here's a quick list
        /// of meanings :
        /// 
        /// - `Unknown`: we don't know if the device has it;
        /// - `Known(None)`: the device doesn't have it;
        /// - `Known(Some(...))`: the device has it, and there's some info 
        ///   available if it's more than a simple button.
        /// 
        /// _Reminder_: for concise handling, `Knowledge<Option<T>>` has the
        /// `is_known_some()` and `is_known_none()` methods.
        #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
        pub struct Capabilities {
            $(
                $(#[$attr])*
                pub $name: state!(cap $ty),
            )+
        }
    };
}

state!{
    /// Some drivers or backend (e.g Linux's `joydev`) are only able 
    /// to report buttons as numbers.
    /// 
    /// In this case, this member is written to, instead of any other.
    /// There's no guarantee about the meaning of each number - it sucks,
    /// but it's up to the game to provide the user with a way to remap
    /// keys as needed.
    /// 
    /// On my Saitek Dual Analog Pad, the mapping is as follows:
    /// 
    /// - 0 => X
    /// - 1 => Y
    /// - 2 => A
    /// - 3 => B
    /// - 4 => L1
    /// - 5 => L2
    /// - 6 => R1
    /// - 7 => R2
    /// - 8 => Back
    /// - 9 => Start
    => numbered_button: numbered_btns,

    #[allow(missing_docs)]
    => a: btn,
    #[allow(missing_docs)]
    => b: btn,
    #[allow(missing_docs)]
    => c: btn,
    #[allow(missing_docs)]
    => x: btn,
    #[allow(missing_docs)]
    => y: btn,
    #[allow(missing_docs)]
    => z: btn,
    #[allow(missing_docs)]
    => l_shoulder: btn,
    #[allow(missing_docs)]
    => r_shoulder: btn,
    #[allow(missing_docs)]
    => l_shoulder_2: btn, // Not on Xbox360
    #[allow(missing_docs)]
    => r_shoulder_2: btn, // Not on Xbox360
    #[allow(missing_docs)]
    => l_trigger: uaxis, // Only on Xbox360
    #[allow(missing_docs)]
    => r_trigger: uaxis, // Only on Xbox360
    #[allow(missing_docs)]
    => select: btn,
    #[allow(missing_docs)]
    => mode: btn,
    #[allow(missing_docs)]
    => start: btn,
    #[allow(missing_docs)]
    => l_thumb: btn,
    #[allow(missing_docs)]
    => r_thumb: btn,
    #[allow(missing_docs)]
    => dpad: Dpad,
    /// The primary (or left, for gamepads) joystick.
    /// 
    /// The Z value is for joysticks that can be moved up or down. It 
    /// does not mean that the joystick is "clicked" (as gamepads allow)
    /// which is what the `l_thumb` is for.
    => l_stick: iaxis3,

    /// See the discussion on `l_stick` above.
    => r_stick: iaxis3,

    #[allow(missing_docs)]
    => throttle: uaxis,
    #[allow(missing_docs)]
    => rudder: iaxis,
    #[allow(missing_docs)]
    => wheel: iaxis,
    #[allow(missing_docs)]
    => hwheel: iaxis,
    #[allow(missing_docs)]
    => gas: uaxis,
    #[allow(missing_docs)]
    => brake: uaxis,

    /// Linux may report a single hat as multiple axis pairs.
    => joystick_hats: hats,
    #[allow(missing_docs)]
    => joystick_trigger: btn,
    #[allow(missing_docs)]
    => joystick_thumb1:  btn,
    #[allow(missing_docs)]
    => joystick_thumb2:  btn,
    #[allow(missing_docs)]
    => joystick_top1:    btn,
    #[allow(missing_docs)]
    => joystick_top2:    btn,
    #[allow(missing_docs)]
    => joystick_pinkie:  btn,
    #[allow(missing_docs)]
    => joystick_dead:    btn
}



macro_rules! buttons {
    ($($(#[$attr:meta])* => $name:ident),+) => {
        #[allow(missing_docs)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub enum ButtonPressed {
            $($(#[$attr])* $name { is_repeat: bool },)+
            /// index from 1 to 6 (MAX_JOYSTICK_BASE_BUTTONS) inclusive
            #[allow(missing_docs)]
            JoystickBase { index: u8, is_repeat: bool },
            /// Some drivers (e.g Linux's `joydev`) are only able to report 
            /// buttons as numbers.
            #[allow(missing_docs)]
            NumberedButton { index: u8, is_repeat: bool },
        }
        #[allow(missing_docs)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub enum ButtonReleased {
            $($(#[$attr])* $name,)+
            /// index from 1 to 6 (MAX_JOYSTICK_BASE_BUTTONS) inclusive
            #[allow(missing_docs)]
            JoystickBase { index: u8 },
            /// Some drivers (e.g Linux's `joydev`) are only able to report 
            /// buttons as numbers.
            #[allow(missing_docs)]
            NumberedButton { index: u8 },
        }
    }
}
buttons!{
    /// A button on Xbox360, Cross on Dualshock
    => A,
    /// B button on Xbox360, Cicle on Dualshock
    => B,
    /// Some controllers (eg. "arcade sticks") have a third button after A and B.
    => C,
    /// X button on Xbox360, Square on Dualshock
    => X,
    /// Y button on Xbox360, Triangle on Dualshock
    => Y,
    /// Some controllers (eg. "arcade sticks") have a third button after X and Y.
    => Z,
    /// Left shoulder button (L1 on Dualshock)
    => LShoulder,
    /// Right shoulder button (R1 on Dualshock)
    => RShoulder,
    /// Second Left shoulder button (L2 on Dualshock)
    /// 
    /// *IMPORTANT*: this will only be fired when it's an actual
    /// *button* and not a *trigger* (as for Xbox360 gamepads).  
    /// *Triggers* are an `AbsoluteAxis`, which means they have a
    /// wider range of values than just "pressed" and "not pressed".
    /// 
    /// If you are planning to handle a generic "L2" button, you should
    /// handle *both* `LShoulder2` and `AbsoluteAxis::LTrigger` (they are
    /// mutually exclusive) and decide a threshold
    /// above which the trigger is "pressed".
    => LShoulder2,
    /// Second Right shoulder button (R2 on Dualshock)
    /// 
    /// See the discussion on `LShoulder2`.
    => RShoulder2,
    /// Back on Xbox360, Select on Dualshock
    => Back,
    /// Menu on Xbox360, Mode on Dualshock
    => Menu,
    /// Start button - some gamepads name it "Pause".
    => Start,
    /// Click on left stick (L3 on Dualshock)
    => LThumb, 
    /// Click on right stick (R3 on Dualshock)
    => RThumb,
    #[allow(missing_docs)]
    => DpadUp,
    #[allow(missing_docs)]
    => DpadDown,
    #[allow(missing_docs)]
    => DpadLeft,
    #[allow(missing_docs)]
    => DpadRight,

    #[allow(missing_docs)]
    => JoystickTrigger,
    #[allow(missing_docs)]
    => JoystickThumb1,
    #[allow(missing_docs)]
    => JoystickThumb2,
    #[allow(missing_docs)]
    => JoystickTop1,
    #[allow(missing_docs)]
    => JoystickTop2,
    #[allow(missing_docs)]
    => JoystickPinkie,
    /// I have no idea what this means, but it's listed
    /// as a possible button on Linux. 
    => JoystickDead
}

#[allow(missing_docs)]
pub type Button = ButtonReleased;

macro_rules! rel_axis {
    ($($(#[$attr:meta])* => $name:ident),+) => {
        /// Relative axis motion events. You'll seldom see them, but they
        /// exist.
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub enum RelativeAxisMotion {
            $($(#[$attr])* $name(IAxis),)+
        }
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        #[allow(missing_docs)]
        pub enum RelativeAxis {
            $($(#[$attr])* $name,)+
        }
    }
}
rel_axis!(
    /// Relative motion of the primary (or left) joystick's X axis.
    => X,
    /// Relative motion of the primary (or left) joystick's Y axis.
    => Y,
    /// Relative motion of the primary (or left) joystick's Z axis.
    => Z,
    /// Relative motion of the secondary (or right) joystick's X axis.
    => RX,
    /// Relative motion of the secondary (or right) joystick's Y axis.
    => RY,
    /// Relative motion of the secondary (or right) joystick's Z axis.
    => RZ,
    /// Relative horizontal wheel motion (I guess?)
    => HWheel,
    /// Relative wheel motion (whatever that means)
    => Wheel,
    /// I have no idea what this means, but Linux provides it
    => Dial
);

macro_rules! abs_axis {
    ($($(#[$attr:meta])* => $name:ident),+) => {
        #[allow(missing_docs)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub enum AbsoluteAxisMotion {
            $($(#[$attr])* $name(IAxis),)+
            /// Absolute joystick hat X-axis motion.
            HatX {
                /// From 0 to MAX_JOYSTICK_HATS inclusive.
                index: u8, 
                #[allow(missing_docs)]
                value: IAxis
            },
            /// Absolute joystick hat Y-axis motion.
            HatY {
                /// From 0 to MAX_JOYSTICK_HATS inclusive.
                index: u8, 
                #[allow(missing_docs)]
                value: IAxis
            },
        }
        #[allow(missing_docs)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub enum AbsoluteAxis {
            $($(#[$attr])* $name,)+
            /// Absolute joystick hat X-axis motion.
            HatX {
                /// From 0 to MAX_JOYSTICK_HATS inclusive.
                index: u8, 
            },
            /// Absolute joystick hat Y-axis motion.
            HatY {
                /// From 0 to MAX_JOYSTICK_HATS inclusive.
                index: u8, 
            },

        }
    }
}
abs_axis!(
    /// Absolute motion of the primary (or left) joystick's X axis.
    => X,
    /// Absolute motion of the primary (or left) joystick's Y axis.
    => Y,
    /// Absolute motion of the primary (or left) joystick's Z axis.
    => Z,
    /// Absolute motion of the secondary (or right) joystick's X axis.
    => RX,
    /// Absolute motion of the secondary (or right) joystick's Y axis.
    => RY,
    /// Absolute motion of the secondary (or right) joystick's Z axis.
    => RZ,
    /// Absolute motion of the left trigger (Xbox360 gamepads).
    => LTrigger,
    /// Absolute motion of the right trigger (Xbox360 gamepads).
    => RTrigger,
    /// Absolute motion of the throttle pedal (steering wheels).
    => Throttle,
    /// Absolute motion of the rudder axis.
    => Rudder,
    /// Absolute motion of the steering wheel.
    => Wheel,
    /// Absolute motion of the gas pedal (steering wheels).
    => Gas,
    /// Absolute motion of the brake pedal (steering wheels).
    => Brake
);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum Error {
    Disconnected,
    SysError(i32),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum JoystickModel {
    Generic,
}
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum SteeringWheelModel {
    Generic,
}
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum GamepadModel {
    Generic,
    Xbox360,
    Dualshock,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum Model {
    Joystick(JoystickModel),
    Gamepad(GamepadModel),
    SteeringWheel(SteeringWheelModel),
    // WiiRemote,
    // Oculus remotes,
    // other devices here...
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum Bus {
    Pci,
    // "Is A Plug n Play" ?
    // Isapnp,
    Usb,
    /// From https://parisc.wiki.kernel.org/index.php/HIL
    /// HIL stands for "Human Interface Loop" and was used on older
    /// PARISC machines and HP300
    /// (m68k processor) series machines to connect keyboards, mice 
    /// and other kind of input devices to the machine.
    Hil,
    Bluetooth,
    Virtual,
}

/// Everything device-specific that is not the device's current state.
/// 
/// Admittedly, games won't be interested in having so much information,
/// however it's useful for later support and adding patches.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct Info<'dev> {
    /// The maximum number of haptic effects playable simultaneously.
    // NOTE: EVIOCGEFFECTS on Linux
    pub max_simultaneous_haptic_effects: Knowledge<u8>,
    /// Which buttons and axes this device reports.
    pub capabilities: Capabilities,
    /// The device's model, if well-known (not that useful).
    pub model: Knowledge<Model>,
    /// The device's GUID, if known.
    pub guid: Knowledge<Guid>,
    /// The device's name (may differ from its `product_name`).
    pub name: Knowledge<&'dev str>,
    /// The device's product name, as reported by the backend.
    pub product_name: Knowledge<&'dev str>, // NOTE: ID_MODEL on udev
    /// The device's product id, as reported by the backend.
    pub product_id: Knowledge<u32>, // NOTE: ID_MODEL_ID on udev
    /// The device's vendor name, as reported by the backend.
    pub vendor_name: Knowledge<&'dev str>,
    /// The device's vendor id, as reported by the backend.
    pub vendor_id: Knowledge<u32>,
    /// The driver version (whatever that means), as reported by the backend.
    pub driver_version: Knowledge<Semver>,
    /// The serial string, as reported by the backend.
    pub serial: Knowledge<&'dev str>,
    /// The time at which the device was connected - this is provided because
    /// the time at which you handle the connection event is always a 
    /// bit late.
    pub plug_time: Knowledge<Instant>,
    /// The device's bus type, as reported by the backend (not that useful).
    pub bus: Knowledge<Bus>,
}
/// A Gamepad, joystick, steering wheel, or whatever device that's reported
/// as a game input device by the backend.
/// 
/// See this module's root documentation for learning how to get and properly
/// use one.
/// 
/// You should drop it when it receives the `Disconnected` event.
#[derive(Debug, Eq, PartialEq)]
pub struct GameInputDevice<'dev> {
    // Private so we return immutable reference through `get_info()`.
    info: Info<'dev>,

    /// The vector of mappings applied to this device.
    /// 
    /// A device has a vector of `Mapping` objects, each acting
    /// as a "conversion pass" for events, applied from first to last.  
    /// You are free to mutate this vector freely. 
    /// Note that this vector can be initially empty, in which case no
    /// memory is allocated.  
    /// 
    /// This is provided because a non-negligible number of devices/drivers
    /// are quirky and require mappings to "fix" their behaviour
    /// (this is the reason why SDL2 provides 
    /// `SDL_GameControllerAddMapping()`,
    /// and also implicitly adds mappings based on its own little
    /// database (`SDL_GameControllerDB`)).  
    /// 
    /// Incidentally, this might be useful to you if you want to allow your
    /// users to remap buttons and axes for your game.  
    pub mappings: Vec<mapping::Mapping>,
}

impl<'dev> GameInputDevice<'dev> {
    /// Queries whether the device is still connected or not.  
    /// If it's not, you may drop this object.
    /// 
    /// You're not required to do this before every query
    /// to the device. The queries themselves return a
    /// `Disconnected` error when appropriate.
    /// 
    /// The returned value is not cached: every call to this function
    /// involves a roundtrip to the backend.
    pub fn is_connected() -> bool {
        unimplemented!()
    }
    /// Query the device's current state (buttons and axes values).
    /// 
    /// This queries the whole state every time, for these reasons:
    /// 
    /// - This is how some backends implement it;
    /// - Providing one method for every possible button or axis
    ///   would clutter this API.
    /// 
    /// The returned state is not cached: every call to this function
    /// involves a roundtrip to the backend.
    pub fn query_state(&self) -> Result<State, Error> {
        unimplemented!()
    }
    /// Query the device's current battery state, if relevant.
    /// 
    /// Some backends or drivers don't support this, in which case
    /// the fields are simply set to `Unknown`.
    pub fn query_battery(&self) -> BatteryState {
        unimplemented!()
    }
    /// Get an immutable reference to the device's `Info` data.
    /// 
    /// This method is cheap because the data is retrieved and stored once
    /// at device initialization time (hence the `get` prefix).
    /// 
    /// Mutation is not allowed because it doesn't make sense for most
    /// of `Info`'s members. Some backends partially implement this,
    /// but I don't see a proper use case for this yet, and it could be
    /// abstracted by your application.
    pub fn info(&self) -> &Info<'dev> {
        &self.info
    }

    /// Uploads an haptic (force feedback) effect.
    pub fn upload_haptic_effect<E: haptic::Effect>(&mut self, _effect: E) 
        -> Result<haptic::UploadedEffect, haptic::Error>
    {
        unimplemented!()
    }
    /// Quickly plays a rumble effect.
    /// 
    /// This shortcut is provided because the `RumbleEffect` is the 
    /// most commonly implemented and used one. Also, on some backends (eg. 
    /// Microsoft's XInput), it is implemented most efficiently this way.
    /// 
    /// There's no way to query the current vibration values
    /// - you have to cache them yourself if you so wish.
    pub fn rumble(&mut self, _rumble: &haptic::RumbleEffect) -> Result<(), haptic::Error> {
        unimplemented!()
    }

    /// Stops early the simple rumble effect played with `rumble()`.
    pub fn stop_rumble(&mut self) -> Result<(), haptic::Error> {
        unimplemented!()
    }

    /// Returns a polling iterator over this device's event queue.
    pub fn poll_iter<'a>(&'a mut self) -> PollIter<'dev, 'a> {
        PollIter { _dev: self }
    }
    /// Returns a waiting iterator over this device's event queue.
    pub fn wait_iter<'a, T: Into<Timeout>>(&'a mut self, timeout: T) -> WaitIter<'dev, 'a> {
        WaitIter { _dev: self, _timeout: timeout.into() }
    }
}

impl<'dev> Drop for GameInputDevice<'dev> {
    fn drop(&mut self) {
        unimplemented!()
    }
}

/// A polling iterator over a device's event queue.
#[derive(Debug)]
pub struct PollIter<'a, 'dev:'a> {
    _dev: &'a mut GameInputDevice<'dev>,
}
/// A waiting iterator over a device's event queue.
#[derive(Debug)]
pub struct WaitIter<'a, 'dev:'a> {
    _dev: &'a mut GameInputDevice<'dev>,
    _timeout: Timeout,
}
/// Events sent by a `GameInputDevice`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    /// The device has been disconnected - you should drop it.
    Disconnected,
    /// A button was pressed.
    ButtonPressed(ButtonPressed),
    /// A button was released.
    ButtonReleased(ButtonReleased),
    /// An absolute axis was moved.
    AbsoluteAxisMotion(AbsoluteAxisMotion),
    /// A relative axis was moved. This kind of event appears to be rare.
    RelativeAxisMotion(RelativeAxisMotion),
}
impl<'a,'dev:'a> Iterator for PollIter<'dev, 'a> {
    type Item = Event;
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
impl<'a,'dev:'a> Iterator for WaitIter<'dev, 'a> {
    type Item = Event;
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}


pub mod monitor {
    //! Monitor-related submodule.

    use super::*;

    #[derive(Debug)]
    /// A polling iterator over a monitor's list of newly connected devices.
    pub struct PollIter<'a> {
        _monitor: &'a mut Monitor,
    }
    #[derive(Debug)]
    /// A waiting iterator over a monitor's list of newly connected devices.
    pub struct WaitIter<'a> {
        _monitor: &'a mut Monitor,
        _timeout: Timeout,
    }
    impl<'a> Iterator for PollIter<'a> {
        type Item = GameInputDevice<'a>;
        fn next(&mut self) -> Option<Self::Item> {
            unimplemented!()
        }
    }
    impl<'a> Iterator for WaitIter<'a> {
        type Item = GameInputDevice<'a>;
        fn next(&mut self) -> Option<Self::Item> {
            unimplemented!()
        }
    }

    /// A hub that listens for incoming game input devices.
    #[derive(Debug)]
    pub struct Monitor;

    impl Monitor {
        /// Creates a monitor, and returns the list of currently connected 
        /// devices.
        /// 
        /// You only need one. Creating more is not forbidden, but wasteful.
        pub fn create<'dev>() -> (Self, Vec<GameInputDevice<'dev>>) {
            unimplemented!()
        }
        /// Returns a polling iterator over the list of newly connected
        /// devices.
        pub fn poll_iter<'a>(&'a mut self) -> PollIter<'a> {
            PollIter { _monitor: self }
        }
        /// Returns a waiting iterator over the list of newly connected
        /// devices.
        pub fn wait_iter<'a, T: Into<Timeout>>(&'a mut self, timeout: T) -> WaitIter<'a> {
            WaitIter { _monitor: self, _timeout: timeout.into() }
        }
    }
}
pub use self::monitor::Monitor;

pub mod mapping {
    //! Remappings for game input devices.

    use super::Event;
    use ::std::collections::HashMap;
    use ::std::str::FromStr;

    /// Remaps buttons and axes to other buttons and axes.
    /// 
    /// There are 3 ways to build one:
    /// 
    /// - The `From` implementation, which directly creates a mapping
    ///   from a `HashMap`;
    /// - The `FromStr` implementation, which attempts to load one
    ///   though a string which follows this module's textual mapping format;
    /// - The `from_sdl2_mapping_str()` method, which is the same as
    ///   the previous one, except it follows SDL2's mapping textual format
    ///   instead.
    ///   
    /// It's up to you to load the strings from files if appropriate.
    // FIXME improve this struct, it's not enough
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Mapping(HashMap<Event, Event>);

    impl FromStr for Mapping {
        type Err = ();
        fn from_str(_s: &str) -> Result<Self, Self::Err> {
            unimplemented!()
        }
    }
    impl Mapping {
        /// See http://wiki.libsdl.org/SDL_GameControllerAddMapping
        pub fn from_sdl2_mapping_str(_s: &str) -> Result<Self,()> {
            unimplemented!()
        }
    }
    // FIXME keep in sync with Mapping's structural changes
    impl From<HashMap<Event,Event>> for Mapping {
        fn from(map: HashMap<Event, Event>) -> Self {
            Mapping(map)
        }
    }
}
pub use self::mapping::Mapping;

pub mod haptic {

    //! Haptic (force feedback)-related data structures.
    //! 
    //! Beware, on Linux, some durations are silently clamped to 
    //! approximately 32 seconds.
    //! Likewise, Linux requires applications to manually clamp some
    //! values - this module does this automatically for you, because
    //! it's an annoying thing to deal with, and failure to do it
    //! supposedly results in undefined behaviour.

    use super::*;
    use ::std::time::Duration;


    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    #[allow(missing_docs)]
    pub enum Error {
        DeviceNotConnected,
        EffectNotSupported,
        /// System-specific error code
        SysError(i32),
    }
    #[derive(Debug, Clone, Eq, PartialEq)]
    /// A haptic effect that was successfully uploaded to a device.
    pub struct UploadedEffect<'a,'dev:'a> {
        _dev: &'a GameInputDevice<'dev>,
        _code: i32,
    }
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    /// A haptic effect that is currently playing. You may want to ignore
    /// this handle when it is handed to you.
    pub struct PlayingEffect<'a,'b:'a,'dev:'b> {
        _fx: &'a UploadedEffect<'b, 'dev>,
    }

    impl<'a,'dev:'a> UploadedEffect<'a, 'dev> {
        /// Attempts to play the effect.
        pub fn play(&self) -> Result<PlayingEffect,()> {
            unimplemented!()
        }
    }
    impl<'a,'b:'a,'dev:'a> PlayingEffect<'a, 'b, 'dev> {
        /// Attempts to stop the effect. You should probably not
        /// rely on this being properly implemented.
        pub fn stop(&self) -> Result<(),()> {
            unimplemented!()
        }
        /// Returns whether the effect is still playing or not.  
        /// If it isn't (or unknown), you should drop this handle.
        pub fn is_playing(&self) -> Knowledge<bool> {
            unimplemented!()
        }
    }
    impl<'a,'dev:'a> Drop for UploadedEffect<'a, 'dev> {
        fn drop(&mut self) {
            unimplemented!()
        }
    }


    // NOTE XXX Read the docs when implementing !
    // For instance some u16 values must be within a certain range.
    
    /// Any haptic (force feedback) effect.
    pub trait Effect {}

    /// The Rumble effect is the simplest one which makes the target
    /// device vibrate.
    ///
    /// Some rumble pads have two motors of different weight.
    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
    pub struct RumbleEffect {
        #[allow(missing_docs)]
        pub duration: Duration,
        #[allow(missing_docs)]
        pub delay_before_start: Duration,
        /// Magnitude for the weak motor, if present.
        /// On Xbox360 controllers, it's the (low-frequency) left motor.
        pub magnitude_for_weak_motor: u16,
        /// Magnitude for the strong motor, if present.
        /// On Xbox360 controllers, it's the (high-frequency) right motor.
        pub magnitude_for_strong_motor: u16,
    }

    /// Envelope, used by some effects
    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
    pub struct Envelope {
        #[allow(missing_docs)]
        pub attack_duration: Duration,
        /// Level at the beginning of the attack
        pub attack_level: u16,
        #[allow(missing_docs)]
        pub fade_duration: Duration,
        /// Level at the end of fade
        pub fade_level: u16,
    }

    /// 
    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
    pub struct ConstantEffect {
        #[allow(missing_docs)]
        pub duration: Duration,
        #[allow(missing_docs)]
        pub delay_before_start: Duration,
        /// Strength of the effect, may be negative
        pub level: i16,
        #[allow(missing_docs)]
        pub envelope: Envelope,
    }
    impl Effect for ConstantEffect {}

    #[allow(missing_docs)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    pub enum Waveform {
        Square,
        Triangle,
        Sine,
        SawUp,
        SawDown,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    #[allow(missing_docs)]
    pub struct PeriodicEffect {
        #[allow(missing_docs)]
        pub duration: Duration,
        #[allow(missing_docs)]
        pub delay_before_start: Duration,
        #[allow(missing_docs)]
        pub waveform: Waveform,
        #[allow(missing_docs)]
        pub period: Duration,
        /// Peak value
        pub magnitude: i16,
        /// Mean of the wave (roughly)
        pub offset: i16,
        /// "Horizontal" shift
        pub phase: u16,
        #[allow(missing_docs)]
        pub envelope: Envelope,
    }
    impl Effect for PeriodicEffect {}

    ///
    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
    pub struct RampEffect {
        #[allow(missing_docs)]
        pub duration: Duration,
        #[allow(missing_docs)]
        pub delay_before_start: Duration,
        #[allow(missing_docs)]
        pub start_level: i16,
        #[allow(missing_docs)]
        pub end_level: i16,
        #[allow(missing_docs)]
        pub envelope: Envelope,
    }
    impl Effect for RampEffect {}

    /// Effect which varies in sync with a button press or axis motion.
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    pub struct ConditionEffect {
        #[allow(missing_docs)]
        pub duration: Duration,
        #[allow(missing_docs)]
        pub delay_before_start: Duration,
        /// Button that triggers the effect, if wanted.
        pub button: Button,
        /// For both directions (left, right), the maximum level when
        /// the joystick is moved all the way to that direction.
        pub saturation: (u16, u16),
        /// How fast the force grows when joystick moves to (left, right)
        pub coeff: (i16, i16),
        /// See the discussion on deadzones.
        pub deadzone_diameter: u16,
        /// See the discussion on deadzones.
        pub deadzone_center: i16,
    }
    impl Effect for ConditionEffect {}
}
pub use self::haptic::{
    UploadedEffect,
    PlayingEffect,
    Effect,
    RumbleEffect,
    Envelope,
    ConstantEffect,
    Waveform,
    PeriodicEffect,
    RampEffect,
    ConditionEffect,
};
