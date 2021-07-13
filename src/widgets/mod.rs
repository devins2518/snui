pub mod button;
pub mod image;
pub mod inner;
pub mod listbox;
pub mod node;
pub mod revealer;
pub mod wbox;

use crate::snui::*;
use std::convert::TryInto;
use std::io::{Write, Seek, SeekFrom, BufWriter};
pub use button::Button;
pub use inner::Inner;
pub use listbox::ListBox;
pub use node::Node;
pub use revealer::Revealer;
pub use self::image::Image;
pub use wbox::Wbox;

const TRANSPARENT: u32 = 0x00_00_00_00;
// const PI: f64 = 3.14159265358979;

pub fn render<C,S>(canvas: &mut C, buffer: &S, x: u32, y: u32)
where
    C: Canvas + Geometry,
    S: Canvas + Geometry
{
    let mut i = 0;
    let buf = buffer.get_buf();
    let width = buffer.get_width() as usize * 4;
    let buf_width = (canvas.get_width() *  4) as usize;
    let mut index = ((x + (y * buffer.get_width())) * 4) as usize;
    while i < buffer.size() && index < canvas.size() {
        let mut writer = BufWriter::new(&mut canvas.get_mut_buf()[index..index+width]);
        writer.write(&buf[i..i+width]).unwrap();
        writer.flush().unwrap();
        i += width;
        index += buf_width;
    }
}

/*
 * The most basic widget one can create. It's the basis of everything else.
 */
#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    width: u32,
    height: u32,
    radius: u32,
    color: Content,
}

impl Geometry for Rectangle {
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        self.height
    }
    fn contains(
        &mut self,
        _widget_x: u32,
        _widget_y: u32,
        _x: u32,
        _y: u32,
        _event: Input,
    ) -> Damage {
        Damage::None
    }
}

impl Drawable for Rectangle {
    fn set_content(&mut self, content: Content) {
        self.color = content;
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
        let surface = Surface::new(self.width,self.height,self.color).unwrap();
        render(canvas, &surface, x, y);
    }
}

impl Widget for Rectangle {}

impl Rectangle {
    pub fn new(width: u32, height: u32) -> Rectangle {
        Rectangle {
            color: Content::Empty,
            width,
            height,
            radius: 0,
        }
    }
    pub fn empty(width: u32, height: u32) -> Rectangle {
        Rectangle {
            color: Content::Empty,
            width,
            height,
            radius: 0,
        }
    }
    pub fn colored(width: u32, height: u32, color: Content) -> Rectangle {
        Rectangle {
            color,
            width,
            height,
            radius: 0,
        }
    }
    pub fn square(size: u32, color: Content) -> Rectangle {
        Rectangle {
            color,
            width: size,
            height: size,
            radius: 0,
        }
    }
}

// A minimal implementation of a canvas widgets can use to draw themselves
#[derive(Clone, Debug)]
pub struct Surface {
    width: u32,
    height: u32,
    canvas: Vec<u8>,
}

impl Geometry for Surface {
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        self.height
    }
    fn contains(
        &mut self,
        _widget_x: u32,
        _widget_y: u32,
        _x: u32,
        _y: u32,
        _event: Input,
    ) -> Damage {
        Damage::None
    }
}

impl Widget for Surface {}

impl Drawable for Surface {
    fn set_content(&mut self, content: Content) {
        match content {
            Content::Byte(byte) => {
                self.canvas = vec![byte; (self.width*self.height*4) as usize]
            }
            Content::Pixel(pixel) => {
                self.canvas.write_all(&pixel.to_ne_bytes()).unwrap();
            }
            Content::Transparent => {
                self.canvas.write_all(&TRANSPARENT.to_ne_bytes()).unwrap();
            }
            _ => {}
        }
        self.canvas.flush().unwrap();
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
        canvas.composite(self, x, y);
    }
}

impl Canvas for Surface {
    fn size(&self) -> usize {
        (self.width * self.height * 4) as usize
    }
    fn damage(&mut self, event: Damage) {
        match event {
            Damage::All { surface } => {
                self.composite(&surface, 0, 0);
            }
            Damage::Area { surface, x, y } => {
                self.composite(&surface, x, y);
            }
            Damage::Destroy {
                x,
                y,
                width,
                height,
            } => {
                self.canvas.write_all(&TRANSPARENT.to_ne_bytes()).unwrap();
                self.canvas.flush().unwrap();
            }
            _ => {}
        }
    }
    fn composite(&mut self, surface: &(impl Canvas + Geometry), x: u32, y: u32) {
        render(self, surface, x, y);
    }
    fn get_buf(&self) -> &[u8] {
        &self.canvas
    }
    fn get_mut_buf(&mut self) -> &mut [u8] {
        &mut self.canvas
    }
}

impl Surface {
    pub fn empty(width: u32, height: u32) -> Surface {
        let canvas = vec![0; (width * height * 4) as usize];
        Surface {
            width,
            height,
            canvas,
        }
    }
    pub fn new(width: u32, height: u32, content: Content) -> Result<Surface, Error> {
        let canvas = match content {
            Content::Byte(byte) => {
                vec![byte; (width * height * 4) as usize]
            }
            Content::Pixel(pixel) => {
                let vec = Vec::new();
                let mut writer = BufWriter::new(vec);
                for _ in 0..width*height {
                    writer.write_all(&pixel.to_ne_bytes()).unwrap();
                }
                writer.flush().unwrap();
                writer.into_inner().unwrap()
            }
            _ => return Err(Error::Null)
        };
        Ok(Surface {
            width,
            height,
            canvas,
        })
    }
}
pub fn to_surface(widget: &(impl Geometry + Drawable)) -> Surface {
    let mut surface = Surface::empty(widget.get_width(), widget.get_height());
    widget.draw(&mut surface, 0, 0);
    surface
}

pub fn border<W: Widget + 'static>(widget: W, gap: u32, background: Content) -> Node {
    let width = widget.get_width() + 2 * gap;
    let height = widget.get_height() + 2 * gap;
    let mut bg = Node::new(Rectangle::colored(width, height, background));
    anchor(&mut bg, widget, Anchor::Center, 0).unwrap();
    bg
}

pub fn anchor<W: 'static, C>(
    container: &mut C,
    widget: W,
    anchor: Anchor,
    margin: u32,
) -> Result<(), Error>
where
    W: Widget,
    C: Container + Geometry,
{
    if container.get_width() >= widget.get_width() && container.get_height() >= widget.get_height()
    {
        let mut x = (container.get_width() - widget.get_width()) / 2;
        let mut y = (container.get_height() - widget.get_height()) / 2;
        match anchor {
            Anchor::Left => x = margin,
            Anchor::Right => x = container.get_width() - widget.get_width() - margin,
            Anchor::Top => y = margin,
            Anchor::Bottom => y = container.get_height() - widget.get_height() - margin,
            Anchor::Center => {}
            Anchor::TopRight => {
                x = container.get_width() - widget.get_width() - margin;
                y = container.get_height() - widget.get_height() - margin;
            }
            Anchor::TopLeft => {
                x = margin;
                y = container.get_height() - widget.get_height() - margin;
            }
            Anchor::BottomRight => {
                x = container.get_width() - widget.get_width() - margin;
                y = margin;
            }
            Anchor::BottomLeft => {
                x = margin;
                y = margin;
            }
        }
        container.put(Inner::new_at(widget, x, y))
    } else {
        Err(Error::Dimension(
            "anchor",
            widget.get_width(),
            widget.get_height(),
        ))
    }
}
