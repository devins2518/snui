pub mod wayland;
pub mod widgets;
pub mod font;

use crate::widgets::Inner;

#[derive(Copy, Clone, Debug)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Clone)]
pub enum Damage<'d> {
    None,
    Hide,
    Destroy,
    Widget {
        widget: Box<&'d dyn Widget>,
        x: u32,
        y: u32,
    }
}

#[derive(Copy, Clone, Debug)]
pub enum State {
    Hide,
    Show,
    Close,
    Same
}

impl<'d> Damage<'d> {
    pub fn is_some(&self) -> bool {
        match self {
            Damage::None => false,
            _ => true
        }
    }
    pub fn new(widget: &'d dyn Widget, x: u32, y: u32) -> Damage<'d> {
        Damage::Widget {
            widget: Box::new(widget),
            x,
            y
        }
    }
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
}

/*
 * Canvas is a trait for types that hold a pixmap.
 */
pub trait Canvas {
    fn size(&self) -> usize;
    fn get_buf(&self) -> &[u8];
    fn get_mut_buf(&mut self) -> &mut [u8];
    fn composite(&mut self, surface: &(impl Canvas + Geometry), x: u32, y: u32);
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
    fn contains<'d>(&'d mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage<'d>;
}

/*
 * A trait for types that be drawn on a Canvas.
 */
pub trait Drawable {
    fn set_color(&mut self, color: u32);
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32);
}

pub trait Widget: Drawable + Geometry {
    // fn name(&self) -> String;
}

pub trait Transform {
    fn scale(&mut self, f: u32);
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
