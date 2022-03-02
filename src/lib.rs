pub mod cache;
pub mod context;
pub mod core;
pub mod mail;
pub mod scene;
#[cfg(feature = "wayland")]
pub mod wayland;
pub mod widgets;

pub use crate::core::*;
use context::*;
use std::ops::{Deref, DerefMut};
use tiny_skia::*;
use widgets::button::Button;
use widgets::layout::Positioner;
use widgets::shapes::WidgetStyle;
use widgets::WidgetBox;

pub mod theme {
    use crate::scene::Texture;
    pub const FG0: u32 = 0xff_C8_BA_A4;
    pub const FG1: u32 = 0xff_cd_c0_ad;
    pub const FG2: u32 = 0xff_be_ae_94;
    pub const BG0: u32 = 0xff_25_22_21;
    pub const BG1: u32 = 0xa0_30_2c_2b;
    pub const BG2: u32 = 0xff_30_2c_2b;
    pub const YELLOW: u32 = 0xff_d9_b2_7c;
    pub const GREEN: u32 = 0xff_95_a8_82;
    pub const BLUE: u32 = 0xff_72_87_97;
    pub const PURPLE: u32 = 0xff_99_83_96;
    pub const BEIGE: u32 = 0xff_ab_93_82;
    pub const ORANGE: u32 = 0xff_d0_8b_65;
    pub const RED: u32 = 0xff_c6_5f_5f;
    pub const TRANSPARENT: Texture = Texture::Transparent;
}

pub fn to_color(color: u32) -> Color {
    let color = color.to_be_bytes();
    Color::from_rgba8(color[3], color[2], color[1], color[0])
}

/// Used to determine damage nodes
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Damage {
    /// Nothing needs to be damaged
    None,
    /// Something needs to be damaged
    Partial,
    /// Damage then request a new frame.
    /// Usefull for animations
    Frame,
}

impl Default for Damage {
    fn default() -> Self {
        Damage::None
    }
}

impl Damage {
    pub fn is_none(&self) -> bool {
        matches!(self, Damage::None)
    }
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
    pub fn upgrade(&mut self) -> Self {
        *self = (*self).max(Damage::Partial);
        *self
    }
}

use std::cmp::Ordering;

impl PartialOrd for Damage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match self {
            Self::None => match other {
                Self::None => Ordering::Equal,
                _ => Ordering::Less,
            },
            Self::Partial => match other {
                Self::None => Ordering::Greater,
                Self::Partial => Ordering::Equal,
                _ => Ordering::Less,
            },
            Self::Frame => match other {
                Self::Frame => Ordering::Equal,
                _ => Ordering::Greater,
            },
        })
    }
}

impl Eq for Damage {}

impl Ord for Damage {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub caps_lock: bool,
    pub logo: bool,
    pub num_lock: bool,
}

impl Modifiers {
    pub fn default() -> Self {
        Modifiers {
            ctrl: false,
            alt: false,
            shift: false,
            caps_lock: false,
            logo: false,
            num_lock: false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Key<'k> {
    pub utf8: Option<&'k String>,
    pub value: &'k [u32],
    pub modifiers: Modifiers,
    pub pressed: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Step {
    Value(f32),
    Increment(i32),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MouseEvent {
    pub pointer: Pointer,
    pub position: Coords,
}

impl MouseEvent {
    pub fn new(position: Coords, pointer: Pointer) -> Event<'static> {
        Event::Pointer(Self { pointer, position })
    }
}

impl Default for MouseEvent {
    fn default() -> Self {
        Self {
            pointer: Pointer::Hover,
            position: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Pointer {
    MouseClick {
        serial: u32,
        button: MouseButton,
        pressed: bool,
    },
    Scroll {
        orientation: Orientation,
        step: Step,
    },
    Hover,
    Enter,
    Leave,
}

impl Pointer {
    pub fn left_button_click(self) -> Option<u32> {
        match self {
            Self::MouseClick {
                serial,
                button,
                pressed,
            } => {
                if button.is_left() && pressed {
                    Some(serial)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn left_button_release(self) -> Option<u32> {
        match self {
            Self::MouseClick {
                serial,
                button,
                pressed,
            } => {
                if button.is_left() && !pressed {
                    Some(serial)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn right_button_click(self) -> Option<u32> {
        match self {
            Self::MouseClick {
                serial,
                button,
                pressed,
            } => {
                if button.is_right() && pressed {
                    Some(serial)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn right_button_release(self) -> Option<u32> {
        match self {
            Self::MouseClick {
                serial,
                button,
                pressed,
            } => {
                if button.is_right() && !pressed {
                    Some(serial)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Extra(u32),
}

impl MouseButton {
    fn new(button: u32) -> MouseButton {
        let button = button % 272;
        match button {
            0 => MouseButton::Left,
            1 => MouseButton::Right,
            2 => MouseButton::Middle,
            _ => MouseButton::Extra(button),
        }
    }
    pub fn is_left(&self) -> bool {
        matches!(self, MouseButton::Left)
    }
    pub fn is_right(&self) -> bool {
        matches!(self, MouseButton::Right)
    }
    pub fn is_extra(self, button: u32) -> bool {
        match self {
            MouseButton::Extra(uint) => uint == button,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Event<'d> {
    /// Sent by the display server when the application needs to be reconfigured
    Configure,
    Focus,
    // Sent on a frame callback with the frame time in ms
    Callback(u32),
    /// Waiting for Wayland-rs 0.3.0 to implement it
    Keyboard(Key<'d>),
    /// Pointer position and type
    Pointer(MouseEvent),
}

impl<'d> Event<'d> {
    pub fn is_cb(&self) -> bool {
        matches!(self, Self::Callback(_))
    }
    pub fn is_configure(&self) -> bool {
        matches!(self, Self::Configure)
    }
}

pub trait Geometry {
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn size(&self) -> Size {
        Size::new(self.width(), self.height())
    }
    fn contains(&self, position: &Coords) -> bool {
        position.x.is_sign_positive()
            && position.y.is_sign_positive()
            && position.x < self.width()
            && position.y < self.height()
    }
}

/// Drawable objects.
///
/// They are given access to the drawing backend.
pub trait Primitive: Geometry {
    fn draw(&self, ctx: &mut DrawContext, transform: tiny_skia::Transform);
}

use scene::{Coords, Scene};

pub trait Widget<T> {
    fn update<'s>(&'s mut self, ctx: &mut UpdateContext<T>) -> Damage;
    fn event<'s>(&'s mut self, ctx: &mut UpdateContext<T>, event: Event<'s>) -> Damage;
    /// The layout is expected to be computed here.
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size;
    fn draw_scene(&mut self, scene: Scene);
}
