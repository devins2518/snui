use crate::widgets::{Inner, Surface};

// This NEEDS to be documentated at some point
#[derive(Copy, Clone, Debug)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

#[derive(Copy, Clone, Debug)]
pub enum Content {
    Empty,
    Transparent,
    Byte(u8),
    Pixel(u32),
    Char(char),
}

#[derive(Clone, Debug)]
pub enum Error {
    Null,
    Overflow(&'static str, u32),
    Dimension(&'static str, u32, u32),
    Message(&'static str),
}

#[derive(Copy, Clone, Debug)]
pub enum Anchor {
    Left,
    Right,
    Top,
    Bottom,
    Center,
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

#[derive(Clone, Debug)]
pub enum Damage {
    Area {
        surface: Surface,
        x: u32,
        y: u32,
    },
    Destroy {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    All {
        surface: Surface,
    },
    None,
    Own,
}

#[derive(Copy, Clone, Debug)]
pub enum Input {
    Key,
    MouseClick {
        time: u32,
        button: u32,
        pressed: bool,
    },
    Enter,
    Leave,
    Hover,
    None,
}

/*
 * Canvas is a trait for types that hold a pixmap.
 */
pub trait Canvas {
    fn damage(&mut self, event: Damage);
    fn get(&self, x: u32, y: u32) -> Content;
    fn set(&mut self, x: u32, y: u32, content: Content);
    fn composite(&mut self, surface: &(impl Canvas + Geometry), x: u32, y: u32);
    fn get_buf(&self) -> &[u8];
}

/*
 * A Container is well.. a widget container.
 * How widgets are contained within the boundaries of it is up to the impelmentation.
 * NOTE: For consistency it's highly recommended to hold widgets within the geometry of a container
 */
pub trait Container {
    fn len(&self) -> u32;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn get_child(&self) -> Result<&dyn Widget, Error>;
    fn add(&mut self, widget: impl Widget + 'static) -> Result<(), Error>;
    fn put(&mut self, widget: Inner) -> Result<(), Error>;
}

pub trait Geometry {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn contains(&mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage;
}

/*
 * A trait for types that be drawn on a Canvas.
 */
pub trait Drawable {
    fn set_content(&mut self, content: Content);
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32);
}

pub trait Widget: Drawable + Geometry {}

pub trait Transform {
    fn scale(&mut self, f: u32);
}

impl Error {
    pub fn debug(&self) {
        match self {
            Error::Dimension(name, w, h) => {
                println!(
                    "requested dimension {}x{} is too large for \"{}\"",
                    w, h, name
                )
            }
            Error::Overflow(name, capacity) => {
                println!("\"{}\" reached its full capacity: {}", name, capacity);
            }
            Error::Message(msg) => println!("{}", msg),
            _ => {}
        }
    }
}
