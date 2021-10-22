pub mod canvas;
pub mod wayland;
pub mod widgets;

use canvas::Canvas;

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

#[derive(Debug)]
pub enum Dispatch {
    Message(&'static str),
    Pointer(f32, f32, Pointer),
    Data(&'static str, Box<dyn std::any::Any + Send + Sync>),
    Keyboard(Key),
    Prepare,
    Commit,
    // Proposal
    // Draw
}

impl Dispatch {
    pub fn data(name: &'static str, data: impl std::any::Any + Send + Sync) -> Self {
        Self::Data(name, Box::new(data))
    }
    pub fn get<T: std::any::Any + Send + Sync>(&self, name: &str) -> Option<&T> {
        match self {
            Dispatch::Data(n, data) => if &name == n {
                data.downcast_ref()
            } else { None }
            _ => None
        }
    }
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
 * A trait for types that can be drawn on a Canvas.
 */
pub trait Drawable {
    fn set_color(&mut self, color: u32);
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32);
}

pub trait Widget: Drawable + Geometry {
    fn damaged(&self) -> bool;
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, canvas: &mut Canvas, dispatch: &Dispatch) -> Option<Damage>;
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
