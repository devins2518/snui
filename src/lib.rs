pub mod context;
pub mod scene;
pub mod wayland;
pub mod widgets;

use context::Context;
use scene::{Background, RenderNode};
use widgets::primitives::{Style, WidgetExt};

pub const FG: u32 = 0xff_C8_BA_A4;

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
pub enum Pointer {
    MouseClick {
        time: u32,
        button: MouseButton,
        pressed: bool,
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event<'d> {
    Commit,
    Prepare,
    Keyboard(Key<'d>),
    Message(&'d str),
    Pointer(f32, f32, Pointer),
}

#[derive(Copy, Clone, Debug)]
pub enum Error {
    Null,
    Overflow(&'static str, u32),
    Dimension(&'static str, f32, f32),
    Message(&'static str),
}

pub trait Container: Geometry {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn add(&mut self, widget: impl Widget + 'static) -> Result<(), Error>;
}

pub trait Geometry {
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)>;
}

pub trait Primitive: Geometry + std::fmt::Debug {
    fn to_background(&self) -> Background;
    fn same(&self, other: &dyn std::any::Any) -> bool;
    fn not_same(&self, other: &dyn std::any::Any) -> bool {
        !self.same(other)
    }
    fn draw(&self, x: f32, y: f32, ctx: &mut Context);
}

pub trait Widget: Geometry {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode;
    fn sync<'d>(&'d mut self, ctx: &mut Context, event: Event);
}

pub fn compare<P: 'static + Primitive + PartialEq>(this: &P, other: &dyn std::any::Any) -> bool {
    if let Some(other) = other.downcast_ref::<P>() {
        return this == other;
    }
    false
}

pub trait Wrapable: Widget + Sized {
    fn wrap(self) -> WidgetExt<Self>;
}
impl<W> Wrapable for W
where
    W: Widget,
{
    fn wrap(self) -> WidgetExt<W> {
        WidgetExt::default(self)
    }
}

impl Error {
    pub fn debug(&self) {
        match self {
            Error::Dimension(name, w, h) => {
                eprintln!(
                    "requested dimension {}x{} is too large for \"{}\"",
                    w, h, name
                )
            }
            Error::Overflow(name, capacity) => {
                eprintln!("\"{}\" reached its full capacity: {}", name, capacity);
            }
            Error::Message(msg) => eprintln!("{}", msg),
            _ => {}
        }
    }
}
