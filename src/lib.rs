pub mod context;
pub mod wayland;
pub mod widgets;

use widgets::Button;
use context::{Context, DamageType};
use widgets::primitives::WidgetShell;

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
pub enum Dispatch<'d> {
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

pub struct Damage<'d> {
    pub widget: &'d dyn Widget,
    pub x: f32,
    pub y: f32,
}

impl<'d> Damage<'d> {
    pub fn new<W: Widget>(x: f32, y: f32, widget: &'d W) -> Damage {
        Damage { widget, x, y }
    }
}

impl<'d> Geometry for Damage<'d> {
    fn width(&self) -> f32 {
        self.widget.width()
    }
    fn height(&self) -> f32 {
        self.widget.height()
    }
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
}

/*
 * A trait for types that can be drawn on a Context.
 */
pub trait Drawable {
    fn set_color(&mut self, color: u32);
    fn draw(&self, context: &mut Context, x: f32, y: f32);
}

pub trait Widget: Drawable + Geometry {
    // fn aware(&self) -> bool;
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, ctx: &mut Context, dispatch: &Dispatch);
}

pub trait Wrapable: Widget + Sized {
    fn wrap(self) -> WidgetShell<Self>;
    fn into_button(self, cb: impl FnMut(&mut Self, Pointer) -> Option<DamageType> + 'static) -> Button<Self>;
}

impl<W> Wrapable for W
where
    W: Widget,
{
    fn wrap(self) -> WidgetShell<W> {
        WidgetShell::default(self)
    }
    fn into_button(self, cb: impl FnMut(&mut W, Pointer) -> Option<DamageType> + 'static) -> Button<Self> {
        Button::new(self, cb)
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
