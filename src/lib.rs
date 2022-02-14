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
use scene::RenderNode;
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
    pub const YEL: u32 = 0xff_d9_b2_7c;
    pub const GRN: u32 = 0xff_95_a8_82;
    pub const BLU: u32 = 0xff_72_87_97;
    pub const PRP: u32 = 0xff_99_83_96;
    pub const BEI: u32 = 0xff_ab_93_82;
    pub const ORG: u32 = 0xff_d0_8b_65;
    pub const RED: u32 = 0xff_c6_5f_5f;
    pub const TRANSPARENT: Texture = Texture::Transparent;
}

pub fn u32_to_source(color: u32) -> Color {
    let color = color.to_be_bytes();
    Color::from_rgba8(color[3], color[2], color[1], color[0])
}

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
        match self {
            Damage::None => true,
            _ => false,
        }
    }
    pub fn is_some(&self) -> bool {
        !self.is_none()
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
        match self {
            MouseButton::Left => true,
            _ => false,
        }
    }
    pub fn is_right(&self) -> bool {
        match self {
            MouseButton::Right => true,
            _ => false,
        }
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
    Configure(&'d [WindowState]),
    /// Sent before a draw
    Draw,
    // Sent on a frame callback with the frame time in ms
    Callback(u32),
    Sync,
    /// Waiting for Wayland-rs 0.3.0 to implement it
    Keyboard(Key<'d>),
    /// Pointer position and type
    Pointer(f32, f32, Pointer),
}

impl<'d> Default for Event<'d> {
    fn default() -> Self {
        Self::Draw
    }
}

impl<'d> Event<'d> {
    pub fn is_cb(&self) -> bool {
        match self {
            Self::Callback(_) => true,
            _ => false,
        }
    }
    pub fn is_configure(&self) -> bool {
        match self {
            Self::Configure(_) => true,
            _ => false,
        }
    }
}

pub trait Geometry {
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn contains(&self, x: f32, y: f32) -> bool {
        scene::Region::new(0., 0., self.width(), self.height()).contains(x, y)
    }
}

/// Drawable objects.
///
/// They are given access to the drawing backend.
pub trait Drawable: Geometry + std::fmt::Debug + DynEq + std::any::Any {
    fn draw(&self, x: f32, y: f32, ctx: &mut DrawContext) {
        self.draw_with_transform_clip(ctx, tiny_skia::Transform::from_translate(x, y), None);
    }
    fn draw_with_clip(
        &self,
        x: f32,
        y: f32,
        ctx: &mut DrawContext,
        clip: Option<&tiny_skia::ClipMask>,
    ) {
        self.draw_with_transform_clip(ctx, tiny_skia::Transform::from_translate(x, y), clip);
    }
    fn draw_with_tranform(
        &self,
        x: f32,
        y: f32,
        ctx: &mut DrawContext,
        tranform: tiny_skia::Transform,
    ) {
        self.draw_with_transform_clip(ctx, tranform.pre_translate(x, y), None);
    }
    fn draw_with_transform_clip(
        &self,
        ctx: &mut DrawContext,
        transform: tiny_skia::Transform,
        clip: Option<&tiny_skia::ClipMask>,
    );
    fn get_texture(&self) -> scene::Texture;
    /// Returns a clone of the primitive with the applied texture.
    fn set_texture(&self, texture: scene::Texture) -> scene::Primitive;
    /// Returns true if the region can fit inside the primitive.
    /// The coordinates will be relative to it
    fn contains(&self, region: &scene::Region) -> bool;
    fn primitive(&self) -> scene::Primitive;
}

pub trait Widget<D>: Geometry {
    /// Creates the render node of the widget.
    fn create_node(&mut self, transform: Transform) -> RenderNode;
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage;
    /// The layout is expected to be computed here.
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size;
}
