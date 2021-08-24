pub mod container;
pub mod image;
pub mod button;

use crate::*;
pub use self::image::Image;
pub use button::Button;
pub use container::{
    border::Border,
    background::Background,
    layout::WidgetLayout,
    Wbox,
};
use std::io::Write;

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
                        let p = [writer[0], writer[1], writer[2], writer[3]];
                        writer.write(&p).unwrap();
                    }
                    255 => {
                        writer.write(&pixel).unwrap();
                    }
                    _ => {
                        let t = pixel[3];
                        let mut p = [writer[0], writer[1], writer[2], writer[3]];
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
    let red = blend_f32(r_a, r_b, t);
    let green = blend_f32(g_a, g_b, t);
    let blue = blend_f32(b_a, b_b, t);
    let alpha = blend_f32(a_a, a_b, t);
    [red as u8, green as u8, blue as u8, alpha as u8]
}

fn blend_f32(a: f32, b: f32, r: f32) -> f32 {
    a + ((b - a) * r)
}

pub fn boxed<W: Widget>(
    widget: W,
    padding: u32,
    border_size: u32,
    bg_color: u32,
    border_color: u32,
) -> Border<Background<W>> {
    let bg = Background::new(widget, bg_color, padding);
    Border::new(bg, border_size, border_color)
}

/*
* The most basic widget one can create. It's the basis of everything else.
*/
#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    width: u32,
    height: u32,
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
        _event: Event,
    ) -> Damage {
        Damage::None
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
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

        if self.color != 0 {
            let mut index = ((x + (y * width as u32)) * 4) as usize;
            for _ in 0..self.height {
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
}

impl Widget for Rectangle {
    fn dispatch<'s>(
        &'s mut self,
        _command: Command,
        _damage_queue: &mut Vec<Damage>,
        _x: u32,
        _y: u32,
    ) {
    }
}

impl Rectangle {
    pub fn new(width: u32, height: u32, color: u32) -> Rectangle {
        Rectangle {
            color,
            width,
            height,
        }
    }
    pub fn empty(width: u32, height: u32) -> Rectangle {
        Rectangle {
            color: 0,
            width,
            height,
        }
    }
    pub fn square(size: u32, color: u32) -> Rectangle {
        Rectangle {
            color,
            width: size,
            height: size,
        }
    }
}

#[derive(Clone)]
pub struct Revealer<N: Widget, R: Widget> {
    normal: N,
    reveal: R,
    state: bool,
    name: Option<String>,
}

impl<N: Widget, R: Widget> Drawable for Revealer<N, R> {
    fn set_color(&mut self, color: u32) {
        if self.state {
            self.reveal.set_color(color);
        } else {
            self.normal.set_color(color);
        }
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        if self.state {
            self.reveal.draw(canvas, width, x, y)
        } else {
            self.normal.draw(canvas, width, x, y)
        }
    }
}

impl<N: Widget, R: Widget> Geometry for Revealer<N, R> {
    fn get_width(&self) -> u32 {
        if self.state {
            self.reveal.get_width()
        } else {
            self.normal.get_width()
        }
    }
    fn get_height(&self) -> u32 {
        if self.state {
            self.reveal.get_height()
        } else {
            self.normal.get_height()
        }
    }
    fn contains<'d>(
        &'d mut self,
        widget_x: u32,
        widget_y: u32,
        x: u32,
        y: u32,
        event: Event,
    ) -> Damage {
        if self.state {
            self.reveal.contains(widget_x, widget_y, x, y, event)
        } else {
            self.normal.contains(widget_x, widget_y, x, y, event)
        }
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        if self.state {
            self.reveal.resize(width, height)
        } else {
            self.normal.resize(width, height)
        }
    }
}

impl<N: Widget, R: Widget> Revealer<N, R> {
    pub fn new(normal: N, reveal: R) -> Revealer<N, R> {
        Revealer {
            state: false,
            normal,
            reveal,
            name: None
        }
    }
    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }
    pub fn toggle(&mut self) {
        if self.state {
            self.state = false
        } else {
            self.state = true
        }
    }
}

impl<N: Widget, R: Widget> Widget for Revealer<N, R> {
    fn dispatch<'s>(
        &'s mut self,
        command: Command,
        damage_queue: &mut Vec<Damage<'s>>,
        x: u32,
        y: u32,
    ) {
        match &self.name {
            Some(name) => if command.eq(name) {
                self.toggle()
            }
            None => {
                if self.state {
                    self.reveal.dispatch(command, damage_queue, x, y)
                } else {
                    self.normal.dispatch(command, damage_queue, x, y)
                }
            }
        }
    }
}
