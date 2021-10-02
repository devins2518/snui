pub mod font;
pub mod wayland;
pub mod widgets;
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, Debug)]
pub enum Error {
    Null,
    Overflow(&'static str, u32),
    Dimension(&'static str, u32, u32),
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
    Data(&'static str, Box<dyn std::any::Any + Send + Sync>),
    Pointer(u32, u32, Pointer),
    Keyboard(Key),
    Commit,
}

impl Dispatch {
    pub fn data<D: std::any::Any + Send + Sync>(name: &'static str, data: D) -> Self {
        Dispatch::Data(name, Box::new(data))
    }
}

pub struct Damage<'d> {
    pub widget: &'d dyn Widget,
    pub x: u32,
    pub y: u32,
}

impl<'d> Damage<'d> {
    pub fn new<W: Widget>(widget: &'d W, x: u32, y: u32) -> Damage {
        Damage {
            widget: widget,
            x,
            y,
        }
    }
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Debug)]
pub struct Canvas<'c> {
    slice: &'c mut [u8],
    pub width: u32,
    pub height: u32,
}

impl<'c> Canvas<'c> {
    fn new(slice: &'c mut [u8], width: u32, height: u32) -> Self {
        Self {
            slice,
            width,
            height,
        }
    }
    fn size(&self) -> usize {
        self.slice.len()
    }
}

impl<'c> Deref for Canvas<'c> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.slice
    }
}

impl<'c> DerefMut for Canvas<'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.slice
    }
}

pub trait Container {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn add(&mut self, widget: impl Widget + 'static) -> Result<(), Error>;
}

pub trait Geometry {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}

/*
 * A trait for types that can be drawn on a Canvas.
 */
pub trait Drawable {
    fn set_color(&mut self, color: u32);
    fn draw(&self, canvas: &mut Canvas, x: u32, y: u32);
}

pub trait Widget: Drawable + Geometry + Send + Sync {
    fn damaged(&self) -> bool;
    fn roundtrip<'d>(
        &'d mut self,
        widget_x: u32,
        widget_y: u32,
        dispatched: &Dispatch,
    ) -> Option<Damage>;
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
