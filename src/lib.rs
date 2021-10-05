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

#[derive(Debug, Copy, Clone)]
pub enum Dispatch {
    Message(&'static str),
    Pointer(u32, u32, Pointer),
    Keyboard(Key),
    Commit,
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

#[derive(Debug, Copy, Clone)]
pub struct DamageReport {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    container: bool,
}

#[derive(Debug)]
pub struct Canvas<'c> {
    slice: &'c mut [u8],
    pub width: u32,
    pub height: u32,
    damage: Vec<DamageReport>
}

impl<'c> Canvas<'c> {
    fn new(slice: &'c mut [u8], width: u32, height: u32) -> Self {
        Self {
            slice,
            width,
            height,
            damage: Vec::new()
        }
    }
    fn size(&self) -> usize {
        self.slice.len()
    }
    pub fn push<W: Geometry>(&mut self, x: u32, y: u32, widget: &W, container: bool) {
        if let Some(last) = self.damage.last() {
            if !(last.container
            && last.x > x
            && last.y > y
            && last.x < x + widget.get_width()
            && last.y < y + widget.get_height())
            {
                self.damage.push(DamageReport {
                    x,
                    y,
                    container,
                    width: widget.get_width(),
                    height: widget.get_height()
                });
            }
        }
    }
    pub fn report(&self) -> &[DamageReport] {
        &self.damage
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
