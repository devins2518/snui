pub mod font;
pub mod wayland;
pub mod widgets;

#[derive(Copy, Clone, Debug)]
pub enum Error {
    Null,
    Overflow(&'static str, u32),
    Dimension(&'static str, u32, u32),
    Message(&'static str),
}

#[derive(Copy, Clone, Debug)]
pub struct Key {
    key: u32,
    modifier: Option<u32>,
    pressed: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum Command<'a> {
    Name(&'a str),
    Key(&'a str, Key),
    Hide,
    Destroy,
    Data(&'a str, &'a dyn std::any::Any),
}

impl<'a> Command<'a> {
    pub fn eq(&self, value: &'a str) -> bool {
        match &self {
            Command::Name(name) => name.eq(&value),
            Command::Key(name, _) => name.eq(&value),
            Command::Data(name, _) => name.eq(&value),
            _ => false,
        }
    }
    pub fn get<T: std::any::Any>(&self) -> Option<&T> {
        match self {
            Command::Data(_, value) => value.downcast_ref(),
            _ => None,
        }
    }
}

pub enum Damage<'d> {
    None,
    Widget {
        widget: Box<&'d dyn Widget>,
        x: u32,
        y: u32,
    },
    Command(Command<'d>),
}

impl<'d> Damage<'d> {
    pub fn is_some(&self) -> bool {
        match self {
            Damage::None => false,
            _ => true,
        }
    }
    pub fn new<W: Widget>(widget: &'d W, x: u32, y: u32) -> Damage {
        Damage::Widget {
            widget: Box::new(widget),
            x,
            y,
        }
    }
    pub fn shift(self, dx: u32, dy: u32) -> Damage<'d> {
        match self {
            Damage::Widget { widget, x, y } => Damage::Widget {
                widget: widget,
                x: x + dx,
                y: y + dy,
            },
            _ => self,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Event {
    MouseClick {
        time: u32,
        button: u32,
        pressed: bool,
    },
    Enter,
    Leave,
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
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn get_child(&self) -> Result<&dyn Widget, Error>;
    fn add(&mut self, widget: impl Widget + 'static) -> Result<(), Error>;
}

pub trait Geometry {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error>;
    fn contains<'d>(
        &'d mut self,
        widget_x: u32,
        widget_y: u32,
        x: u32,
        y: u32,
        event: Event,
    ) -> Damage;
}

/*
 * A trait for types that can be drawn on a Canvas.
 */
pub trait Drawable {
    fn set_color(&mut self, color: u32);
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32);
}

pub trait Widget: Drawable + Geometry + Send + Sync {
    fn dispatch<'s>(
        &'s mut self,
        command: Command,
        damage_queue: &mut Vec<Damage<'s>>,
        x: u32,
        y: u32,
    );
}

pub trait Transform {
    fn scale(&mut self, f: f32);
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
