pub mod action;
pub mod image;
pub mod singles;
pub mod revealer;
pub mod wbox;

use crate::*;
use std::io::Write;
pub use singles::{
    Inner, Border, Rectangle, Background
};
pub use revealer::Revealer;
pub use self::image::Image;
pub use wbox::{Wbox, Alignment};
pub use action::{Button, Actionnable};

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

pub fn boxed<W: Widget + Clone>(widget: W, padding: u32, border_size: u32, bg_color: u32, border_color: u32) -> Border<Background<Rectangle, W>> {
	let bg = Background::new(widget, Rectangle::new(1,1, bg_color), padding);
	Border::new(bg, border_size, border_color)
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

