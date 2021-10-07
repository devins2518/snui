pub mod wayland;
pub mod widgets;
use euclid::default::{Box2D, Point2D};
use raqote::*;
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

pub struct Canvas {
    target: DrawTarget,
    damage: Vec<DamageReport>,
}

impl Canvas {
    fn new(width: u32, height: u32) -> Self {
        Self {
            target: DrawTarget::new(width as i32, height as i32),
            damage: Vec::new(),
        }
    }
    fn target(&mut self) -> &mut DrawTarget {
        &mut self.target
    }
    fn size(&self) -> usize {
        self.target.get_data_u8().len()
    }
    pub fn push<W: Geometry>(&mut self, x: u32, y: u32, widget: &W, container: bool) {
        if let Some(last) = self.damage.last() {
            if !(last.container
                && last.x > x
                && last.y > y
                && last.x < x + widget.width()
                && last.y < y + widget.height())
            {
                self.damage.push(DamageReport {
                    x,
                    y,
                    container,
                    width: widget.width(),
                    height: widget.height(),
                });
            }
        }
    }
    pub fn report(&self) -> &[DamageReport] {
        &self.damage
    }
}

impl Geometry for Canvas {
    fn width(&self) -> u32 {
        self.target.width() as u32
    }
    fn height(&self) -> u32 {
        self.target.height() as u32
    }
}

impl Drawable for Canvas {
    fn set_color(&mut self, color: u32) {
        let color = color.to_be_bytes();
        self.target.fill_rect(
            0.,
            0.,
            self.target.width() as f32,
            self.target.height() as f32,
            &Source::Solid(SolidSource {
                a: color[0],
                r: color[1],
                g: color[2],
                b: color[3],
            }),
            &DrawOptions::new()
        )
    }
    fn draw(&self, canvas: &mut Canvas, x: u32, y: u32) {
        for damage in &self.damage {
            canvas.damage.push(*damage);
        }
        canvas.target().blend_surface(
            &self.target,
            Box2D::new(
                euclid::point2(x as i32, y as i32),
                euclid::point2(
                    (x + self.width()) as i32,
                    (y + self.height()) as i32
                )
            ),
            Point2D::new(x as i32, y as i32),
            BlendMode::Add)
    }
}

impl Deref for Canvas {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.target.get_data_u8()
    }
}

impl DerefMut for Canvas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.target.get_data_u8_mut()
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
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

/*
 * A trait for types that can be drawn on a Canvas.
 */
pub trait Drawable {
    fn set_color(&mut self, color: u32);
    fn draw(&self, canvas: &mut Canvas, x: u32, y: u32);
}

pub trait Widget: Drawable + Geometry {
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
