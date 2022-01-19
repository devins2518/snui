pub mod context;
pub mod controller;
pub mod font;
pub mod scene;
pub mod widgets;
#[cfg(feature = "wayland")]
pub mod wayland;

use context::*;
use scene::RenderNode;
pub use tiny_skia::*;
use widgets::button::{Button, Proxy};
use widgets::container::Child;
use widgets::shapes::WidgetExt;
use widgets::{Padding, WidgetBox};

pub mod style {
    use crate::scene::Background;
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
    pub const TRANSPARENT: Background = Background::Transparent;
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
    /// Damage then request a new frame
    Frame,
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
pub enum Move {
    Value(f32),
    Step(i32),
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
        value: Move,
    },
    Hover,
    Enter,
    Leave,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WindowState {
    Maximized,
    Resizing,
    Fullscreen,
    /// Client window decorations should be painted as if the window is active.
    Activated,
    Deactivated,
    TiledLeft,
    TiledRight,
    TiledBottom,
    TiledTop
}

#[derive(Debug, PartialEq)]
pub enum Event<'d, M> {
    /// Sent when a full redraw is neccessary
    Configure(WindowState),
    /// Doesn't have any particular meaning
    Prepare,
    // Sent on a frame callback with the frame time in ms
    Callback(u32),
    /// A reference to your message
    Message(&'d M),
    /// Waiting for Wayland-rs 0.3.0 to implement it
    Keyboard(Key<'d>),
    /// Pointer position and type
    Pointer(f32, f32, Pointer),
}

impl<'d, M> Clone for Event<'d, M> {
    fn clone(&self) -> Self {
        match self {
            Self::Prepare => Self::Prepare,
            Self::Configure(state) => Self::Configure(*state),
            Self::Callback(ft) => Self::Callback(*ft),
            Self::Message(msg) => Self::Message(*msg),
            Self::Keyboard(key) => Self::Keyboard(*key),
            Self::Pointer(x, y, p) => Self::Pointer(*x, *y, *p),
        }
    }
}

impl<'d, M> Default for Event<'d, M> {
    fn default() -> Self {
        Self::Prepare
    }
}

impl<'d, M> Copy for Event<'d, M> {}

impl<'d, M> Event<'d, M> {
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
    fn set_width(&mut self, _width: f32) -> Result<(), f32> {
        Err(self.width())
    }
    fn set_height(&mut self, _height: f32) -> Result<(), f32> {
        Err(self.height())
    }
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        let err_width = self.set_width(width);
        let err_height = self.set_height(height);

        if let Err(width) = err_width {
            if let Err(height) = err_height {
                Err((width, height))
            } else {
                Err((width, height))
            }
        } else {
            if let Err(height) = err_height {
                Err((width, height))
            } else {
                Ok(())
            }
        }
    }
    fn create_canvas(&self, x: f32, y: f32) -> context::canvas::Canvas {
        context::canvas::Canvas::new(scene::Region::new(x, y, self.width(), self.height()))
    }
    fn contains(&self, x: f32, y: f32) -> bool {
        scene::Region::new(0., 0., self.width(), self.height()).contains(x, y)
    }
}

pub trait Primitive: Geometry + std::fmt::Debug {
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
    fn get_background(&self) -> scene::Background;
    fn apply_background(&self, background: scene::Background) -> scene::PrimitiveType;
    /// Tell if the region can fit inside the Primitive.
    /// The coordinates will be relative to it
    fn contains(&self, region: &scene::Region) -> bool;
    // Basically Clone
    fn into_primitive(&self) -> scene::PrimitiveType;
}

pub trait Widget<M>: Geometry {
    /// Creates the RenderNode of the widget.
    /// Widgets are expected to compute their layout when this method is invoked.
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode;
    /// Interface to communicate with the controller and retained mode draw operation
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage;
}

pub trait Flex<G>: Geometry + Sized {
    fn with_width(self, width: f32) -> Self;
    fn with_height(self, height: f32) -> Self;
    fn with_size(self, width: f32, height: f32) -> Self;
}

pub trait WidgetUtil<M>: Widget<M> + Sized {
    fn ext(self) -> WidgetExt<M, Self>;
    fn clamp(self) -> WidgetBox<M, Self>;
    fn pad(self, padding: f32) -> Padding<M, Self>;
    fn child(self) -> Child<M>;
    fn button<F: for<'d> FnMut(&'d mut Proxy<M, Self>, &'d mut SyncContext<M>, Pointer)>(
        self,
        cb: F,
    ) -> Button<M, Self, F>;
}

impl<G> Flex<G> for G
where
    G: Geometry,
{
    fn with_width(mut self, width: f32) -> Self {
        let _ = self.set_width(width);
        self
    }
    fn with_height(mut self, height: f32) -> Self {
        let _ = self.set_height(height);
        self
    }
    fn with_size(mut self, width: f32, height: f32) -> Self {
        let _ = self.set_size(width, height);
        self
    }
}

impl<W, M> WidgetUtil<M> for W
where
    W: Widget<M> + 'static,
{
    fn pad(self, padding: f32) -> Padding<M, Self> {
        Padding::new(self).even_padding(padding)
    }
    fn clamp(self) -> WidgetBox<M, Self> {
        WidgetBox::new(self)
    }
    fn ext(self) -> WidgetExt<M, Self> {
        WidgetExt::new(self)
    }
    fn child(self) -> Child<M> {
        Child::new(self)
    }
    fn button<F>(self, cb: F) -> Button<M, Self, F>
    where
        F: for<'d> FnMut(&'d mut Proxy<M, W>, &'d mut SyncContext<M>, Pointer),
    {
        Button::new(self, cb)
    }
}
