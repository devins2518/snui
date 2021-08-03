pub mod button;
pub mod image;
pub mod node;
pub mod revealer;
pub mod wbox;

use crate::*;
use std::rc::Rc;
use std::io::Write;
pub use node::Node;
pub use revealer::Revealer;
pub use self::image::Image;
pub use wbox::{Wbox, Alignment};
pub use button::{Button, Actionnable};

// For rounded corners eventually
// const PI: f64 = 3.14159265358979;

pub fn render<S>(canvas: &mut [u8], buffer: &S, mut width: usize, x: u32, y: u32)
where
    S: Canvas + Geometry,
{
    let mut index = ((x + (y * width as u32)) * 4) as usize;
    width *= 4;
    for buf in buffer.get_buf().chunks(buffer.get_width() as usize * 4) {
        if index >= canvas.len() {
            break;
        } else {
            let mut writer = &mut canvas[index..];
            for pixel in buf.chunks(4) {
                match pixel[3] {
                    0 => {
                        let p = [writer[0],writer[1],writer[2],writer[3]];
                        writer.write(&p).unwrap();
                    }
                    255 => {
                        writer.write(&pixel).unwrap();
                    }
                    _ => {
                        let t = pixel[3];
                        let mut p = [writer[0],writer[1],writer[2],writer[3]];
                        p = blend(&pixel, &p, (255 - t) as f32 / 255.0);
                        writer.write(&p).unwrap();
                    }
                }
            }
            index += width;
        }
    }
}
pub fn blend(pix_a: &[u8], pix_b: &[u8], t: f32) -> [u8; 4] {
    let (r_a, g_a, b_a, a_a) = (
        pix_a[0] as f32,
        pix_a[1] as f32,
        pix_a[2] as f32,
        pix_a[3] as f32,
    );
    let (r_b, g_b, b_b, a_b) = (
        pix_b[0] as f32,
        pix_b[1] as f32,
        pix_b[2] as f32,
        pix_b[3] as f32,
    );
    let red   = blend_f32(r_a, r_b, t);
    let green = blend_f32(g_a, g_b, t);
    let blue  = blend_f32(b_a, b_b, t);
    let alpha = blend_f32(a_a, a_b, t);
    [red as u8, green as u8, blue as u8, alpha as u8]
}

fn blend_f32(a: f32, b: f32, r: f32) -> f32 {
    a + ((b - a) * r)
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
    fn resize(&mut self, width: u32, height: u32) -> Result<(),Error> {
        self.width = width;
        self.height = height;
        Ok(())
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

impl Widget for Rectangle {
    fn action<'s>(&'s mut self, _name: Action, _event_loop: &mut Vec<Damage>, _widget_x: u32, _widget_y: u32) {}
}

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

#[derive(Clone)]
pub struct Inner {
    x: u32,
    y: u32,
    mapped: bool,
    anchor: Anchor,
    child: Rc<dyn Widget>,
}

impl Geometry for Inner {
    fn get_width(&self) -> u32 {
        self.child.as_ref().get_width()
    }
    fn get_height(&self) -> u32 {
        self.child.as_ref().get_height()
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(),Error> {
        Rc::get_mut(&mut self.child).unwrap().resize(width, height)
    }
    fn contains<'d>(&'d mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        if x > widget_x
            && y > widget_y
            && x < widget_x + self.get_width()
            && y < widget_y + self.get_height()
        {
            Rc::get_mut(&mut self.child).unwrap().contains(widget_x, widget_y, x, y, event)
        } else {
            Damage::None
        }
    }
}

impl Container for Inner {
    fn len(&self) -> u32 {
        1
    }
    fn add(&mut self, _widget: impl Drawable + 'static) -> Result<(), Error> {
        Err(Error::Overflow("inner", 1))
    }
    fn put(&mut self, _widget: Inner) -> Result<(), Error> {
        Err(Error::Overflow("inner", 1))
    }
    fn get_child(&self) -> Result<&dyn Widget, Error> {
        Err(Error::Message("get_child is not valid on \"inner\""))
    }
}

impl Drawable for Inner {
    fn set_color(&mut self, color: u32) {
        Rc::get_mut(&mut self.child).unwrap().set_color(color)
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.child.as_ref().draw(canvas, width, x, y);
    }
}

impl Widget for Inner {
    fn action<'s>(&'s mut self, name: Action, event_loop: &mut Vec<Damage>, widget_x: u32, widget_y: u32) {
        Rc::get_mut(&mut self.child).unwrap().action(name, event_loop, widget_x, widget_y);
    }
}

impl Inner {
    pub fn new(child: impl Widget + 'static) -> Inner {
        Inner {
            x: 0,
            y: 0,
            mapped: false,
            anchor: Anchor::TopLeft,
            child: Rc::new(child),
        }
    }
    pub fn new_at(child: impl Widget + 'static, anchor: Anchor, x:  u32, y: u32) -> Inner {
        Inner {
            x,
            y,
            anchor,
            mapped: false,
            child: Rc::new(child),
        }
    }
    pub fn get_anchor(&self) -> Anchor {
        self.anchor
    }
    pub fn is_mapped(&self) -> bool {
        self.mapped
    }
    pub fn map(&mut self) {
        self.mapped = true;
    }
    pub fn unmap(&mut self) {
        self.mapped = false;
    }
    pub fn coords(&self) -> (u32, u32) {
        (self.x, self.y)
    }
    pub fn get_location(&self, width: u32, height: u32) -> Result<(u32, u32), Error> {
        Ok(match self.anchor {
            Anchor::Left => (self.x, self.y),
            Anchor::Right => (
                width - self.get_width() - self.x,
                (height - self.get_height() + self.y)/2
            ),
            Anchor::Top => ((width - self.get_width() + self.x)/2, self.y),
            Anchor::Bottom => ((width - self.get_width() + self.x)/2, height - self.y - self.get_height()),
            Anchor::Center => (
                (width - self.get_width() + self.x)/2,
                (height - self.get_height() + self.y)/2,
            ),
            Anchor::TopRight => (width - self.x - self.get_width(), self.y),
            Anchor::TopLeft => (self.x, self.y),
            Anchor::BottomRight => (
                width - self.x - self.get_width(),
                height - self.y - self.get_height()
            ),
            Anchor::BottomLeft => (
                self.x,
                height - self.y - self.get_height()
            )
        })
    }
    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
    }
    pub fn set_location(&mut self, x: u32, y: u32) {
        self.x = x;
        self.y = y;
    }
    pub fn translate(&mut self, x: u32, y: u32) {
        self.x += x;
        self.y += y;
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
    fn resize(&mut self, width: u32, height: u32) -> Result<(),Error> {
        self.width = width;
        self.height = height;
        Ok(())
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

impl Widget for Surface {
    fn action<'s>(&'s mut  self, _name: Action, _event_loop: &mut Vec<Damage>, _widget_x: u32, _widget_y: u32) {}
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
    anchor(&mut bg, widget, Anchor::Center, 0, 0).unwrap();
    bg
}

pub fn anchor<W: 'static, C>(
    container: &mut C,
    widget: W,
    anchor: Anchor,
    x: u32,
    y: u32,
) -> Result<(), Error>
where
    W: Widget,
    C: Container + Geometry,
{
    if container.get_width() >= widget.get_width() && container.get_height() >= widget.get_height()
    {
        container.put(Inner::new_at(widget, anchor, x, y))
    } else {
        Err(Error::Dimension(
            "anchor",
            widget.get_width(),
            widget.get_height(),
        ))
    }
}
