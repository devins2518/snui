use crate::widgets::Surface;

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
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
    Center,
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

// API to manipulate the Canvas
pub trait Canvas {
    fn display(&mut self);
    fn damage(&mut self, event: Damage);
    fn get(&self, x: u32, y: u32) -> Content;
    fn set(&mut self, x: u32, y: u32, content: Content);
    fn composite(&mut self, surface: &(impl Canvas + Geometry), x: u32, y: u32);
}

// Make all containers hold Inner or rename it
// Knowing a Drawable's position is essential for damage tracking
pub trait Container {
    fn len(&self) -> u32;
    // fn get_child(&self) -> Vec<&(Drawable + Geometry)>;
    fn add(&mut self, object: impl Widget + 'static) -> Result<(), Error>;
}

pub trait Geometry {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_location(&self) -> (u32, u32);
    fn set_location(&mut self, x: u32, y: u32);
    fn contains(&mut self, x: u32, y: u32, event: Input) -> Damage;
}

pub trait Drawable {
    fn set_content(&mut self, content: Content);
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32);
}

pub trait Widget: Drawable + Geometry {}

pub fn to_surface(widget: &(impl Geometry + Drawable)) -> Surface {
    let mut surface = Surface::new(
        widget.get_width(),
        widget.get_height(),
        Content::Empty,
    );
    widget.draw(&mut surface, 0, 0);
    surface
}

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
