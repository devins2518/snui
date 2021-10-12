pub mod container;
pub mod image;
pub mod text;
pub mod primitives;

pub use self::image::Image;
use crate::*;
pub use container::{layout::WidgetLayout, Wbox};
use std::io::Write;

pub fn render(canvas: &mut Canvas, buffer: &[u8], width: f32, x: f32, y: f32) {
    let stride = canvas.width() as usize * 4;
    let mut index = ((x + (y * canvas.width())) * 4.) as usize;
    for buf in buffer.chunks(width as usize * 4) {
        if index >= canvas.len() {
            break;
        } else {
            let mut writer = &mut canvas[index..];
            for pixel in buf.chunks(4) {
                match pixel[3] {
                    0 => {
                        writer
                            .write(&[writer[0], writer[1], writer[2], writer[3]])
                            .unwrap();
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
            index += stride;
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

pub struct Actionnable<W: Geometry + Drawable> {
    widget: W,
    focused: bool,
    cb: Box<dyn FnMut(&mut W, Dispatch, f32, f32) -> Option<Damage>>,
}

impl<W: Widget> Actionnable<W> {
    pub fn new(
        widget: W,
        cb: impl FnMut(&mut W, Dispatch, f32, f32) -> Option<Damage> + 'static,
    ) -> Self {
        Self {
            widget,
            focused: false,
            cb: Box::new(cb),
        }
    }
}

impl<W: Widget> Geometry for Actionnable<W> {
    fn width(&self) -> f32 {
        self.widget.width()
    }
    fn height(&self) -> f32 {
        self.widget.height()
    }
}

impl<W: Widget> Drawable for Actionnable<W> {
    fn set_color(&mut self, color: u32) {
        self.widget.set_color(color);
    }
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32) {
        self.widget.draw(canvas, x, y)
    }
}

impl<W: Widget> Widget for Actionnable<W> {
    fn damaged(&self) -> bool {
        self.widget.damaged()
    }
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, dispatch: &Dispatch) -> Option<Damage> {
        if let Dispatch::Pointer(x, y, pointer) = dispatch {
            if *x > wx && *y > wy && *x < wx + self.width() && *y < wy + self.height() {
                if !self.focused {
                    self.focused = true;
                    (self.cb)(
                        &mut self.widget,
                        Dispatch::Pointer(*x, *y, Pointer::Enter),
                        wx,
                        wy,
                    )
                } else {
                    (self.cb)(
                        &mut self.widget,
                        Dispatch::Pointer(*x, *y, *pointer),
                        wx,
                        wy,
                    )
                }
            } else if self.focused {
                self.focused = false;
                (self.cb)(
                    &mut self.widget,
                    Dispatch::Pointer(*x, *y, Pointer::Leave),
                    wx,
                    wy,
                )
            } else {
                None
            }
        } else {
            (self.cb)(&mut self.widget, *dispatch, wx, wy)
        }
    }
}
