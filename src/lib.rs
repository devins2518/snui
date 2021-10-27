pub mod context;
pub mod wayland;
pub mod widgets;

use context::{Context, Dispatch};

#[derive(Copy, Clone, Debug)]
pub enum Error {
    Null,
    Overflow(&'static str, u32),
    Dimension(&'static str, f32, f32),
    Message(&'static str),
}

#[derive(Copy, Clone, Debug)]
pub struct Key {
    pub value: u32,
    modifier: Option<u32>,
    pressed: bool,
}

pub struct Damage<'d> {
    pub widget: &'d dyn Widget,
    pub x: f32,
    pub y: f32,
}

impl<'d> Damage<'d> {
    pub fn new<W: Widget>(x: f32, y: f32, widget: &'d W) -> Damage {
        Damage {
            widget: widget,
            x,
            y,
        }
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Pointer {
    MouseClick {
        time: u32,
        button: u32,
        pressed: bool,
    },
    Hover,
    Enter,
    Leave,
}
pub trait Container {
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
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, context: &mut Context, dispatch: &Dispatch);
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
