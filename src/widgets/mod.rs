pub mod button;
pub mod image;
pub mod inner;
pub mod node;
pub mod revealer;
pub mod wbox;

pub use self::image::Image;
use crate::*;
pub use button::Button;
pub use inner::Inner;
pub use node::Node;
pub use revealer::Revealer;
use std::io::Write;
pub use wbox::{Wbox, Alignment};

const TRANSPARENT: u32 = 0x00_00_00_00;
// For rounded corners eventually
// const PI: f64 = 3.14159265358979;

pub fn render<S>(canvas: &mut [u8], buffer: &S, mut width: usize, x: u32, y: u32)
where
    S: Canvas + Geometry,
{
    let buf = buffer.get_buf();
    let buf_width = buffer.get_width() as usize * 4;
    let mut index = ((x + (y * width as u32)) * 4) as usize;
    width *= 4;
    for i in (0..buf.len()).into_iter().step_by(buf_width) {
        if index >= canvas.len() {
            break;
        } else {
            let mut writer = &mut canvas[index..];
            if i + buf_width < buf.len() {
                writer.write(&buf[i..i + buf_width]).unwrap();
            } else {
                writer.write(&buf[i..]).unwrap();
            }
            writer.flush().unwrap();
            index += width;
        }
    }
}
pub fn blend(pix_a: &[u8], pix_b: &[u8], t: i32) -> [u8; 4] {
    let (r_a, g_a, b_a, a_a) = (
        pix_a[0] as i32,
        pix_a[1] as i32,
        pix_a[2] as i32,
        pix_a[3] as i32,
    );
    let (r_b, g_b, b_b) = (
        pix_b[0] as i32,
        pix_b[1] as i32,
        pix_b[2] as i32,
    );
    let red   = (r_a * (255 - t) + r_b * t) / 255;
    let green = (g_a * (255 - t) + g_b * t) / 255;
    let blue  = (b_a * (255 - t) + b_b * t) / 255;
    let alpha = 255 - ((255 - a_a) * (255 - t) / 255);
    [red as u8, green as u8, blue as u8, alpha as u8]
}

/*
 * The most basic widget one can create. It's the basis of everything else.
 */
#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    width: u32,
    height: u32,
    radius: u32,
    color: u32,
}

impl Geometry for Rectangle {
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        self.height
    }
    fn contains<'d>(
        &'d mut self,
        _widget_x: u32,
        _widget_y: u32,
        _x: u32,
        _y: u32,
        _event: Input,
    ) -> Damage<'d> {
        Damage::None
    }
}

impl Drawable for Rectangle {
    fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        let buf = self.color.to_ne_bytes();

        let mut index = ((x + (y * width as u32)) * 4) as usize;
        for _ in 0.. self.height {
            if index >= canvas.len() {
                break;
            } else {
                let mut writer = &mut canvas[index..];
                for _ in 0..self.width {
                    writer.write_all(&buf).unwrap();
                }
                writer.flush().unwrap();
                index += width as usize * 4;
            }
        }
    }
}

impl Widget for Rectangle {}

impl Rectangle {
    pub fn new(width: u32, height: u32, color: u32) -> Rectangle {
        Rectangle {
            color,
            width,
            height,
            radius: 0,
        }
    }
    pub fn empty(width: u32, height: u32) -> Rectangle {
        Rectangle {
            color: 0,
            width,
            height,
            radius: 0,
        }
    }
    pub fn square(size: u32, color: u32) -> Rectangle {
        Rectangle {
            color,
            width: size,
            height: size,
            radius: 0,
        }
    }
    pub fn set_radius(&mut self, radius: u32) {
        self.radius = radius;
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
    fn contains<'d>(
        &'d mut self,
        _widget_x: u32,
        _widget_y: u32,
        _x: u32,
        _y: u32,
        _event: Input,
    ) -> Damage<'d> {
        Damage::None
    }
}

impl Widget for Surface {}

impl Drawable for Surface {
    fn set_color(&mut self, color: u32) {
        self.canvas.write_all(&color.to_ne_bytes()).unwrap();
        self.canvas.flush().unwrap();
    }
    fn draw(&self, canvas: &mut [u8], _width: u32, x: u32, y: u32) {
        render(canvas, self, self.get_width() as usize, x, y);
    }
}

impl Canvas for Surface {
    fn size(&self) -> usize {
        (self.width * self.height * 4) as usize
    }
    fn composite(&mut self, surface: &(impl Canvas + Geometry), x: u32, y: u32) {
        let width = self.get_width();
        render(self.get_mut_buf(), surface, width as usize, x, y);
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
    pub fn new(width: u32, height: u32, color: u32) -> Result<Surface, Error> {
        let canvas =  {
                let mut vec = Vec::new();
                for _ in 0..width * height {
                    vec.write_all(&color.to_ne_bytes()).unwrap();
                }
                vec.flush().unwrap();
                vec
        };
        Ok(Surface {
            width,
            height,
            canvas,
        })
    }
    pub fn from(canvas: Vec<u8>, width: u32, height: u32) -> Surface {
        Surface {
            canvas,
            width,
            height
        }
    }
}
pub fn to_surface(widget: &(impl Geometry + Drawable)) -> Surface {
    let mut surface = Surface::empty(widget.get_width(), widget.get_height());
    let width = surface.get_width();
    widget.draw(surface.get_mut_buf(), width, 0, 0);
    surface
}

pub fn border<W: Widget + 'static>(widget: W, gap: u32, background: u32) -> Node {
    let width = widget.get_width() + 2 * gap;
    let height = widget.get_height() + 2 * gap;
    let mut bg = Node::new(Rectangle::new(width, height, background));
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
                y = margin;
            }
            Anchor::TopLeft => {
                x = margin;
                y = margin;
            }
            Anchor::BottomRight => {
                x = container.get_width() - widget.get_width() - margin;
                y = container.get_height() - widget.get_height() - margin;
            }
            Anchor::BottomLeft => {
                x = margin;
                y = container.get_height() - widget.get_height() - margin;
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
