pub mod button;
pub mod container;
pub mod image;
pub mod label;

use crate::*;
use std::io::Write;
pub use button::Button;
pub use self::image::Image;
pub use container::{layout::WidgetLayout, Wbox};
pub use rectangle::*;

pub fn render(canvas: &mut Canvas, buffer: &[u8], width: u32, x: u32, y: u32) {
    let stride = canvas.width as usize * 4;
    let mut index = ((x + (y * canvas.width as u32)) * 4) as usize;
    for buf in buffer.chunks(width as usize * 4) {
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

pub mod rectangle {
    use crate::*;
    use std::io::Write;

    #[derive(Copy, Clone, Debug)]
    pub struct Rectangle {
        width: u32,
        height: u32,
        color: u32,
        damaged: bool,
    }

    impl Geometry for Rectangle {
        fn get_width(&self) -> u32 {
            self.width
        }
        fn get_height(&self) -> u32 {
            self.height
        }
    }

    impl Drawable for Rectangle {
        fn set_color(&mut self, color: u32) {
            self.color = color;
        }
        fn draw(&self, canvas: &mut Canvas, x: u32, y: u32) {
            if self.color != 0 {
                let buf = self.color.to_ne_bytes();
                let stride = canvas.width as usize * 4;

                let mut index = ((x + (y * canvas.width as u32)) * 4) as usize;
                for _ in 0..self.height {
                    if index >= canvas.len() {
                        break;
                    } else {
                        let mut writer = &mut canvas[index..];
                        for _ in 0..self.width {
                            writer.write_all(&buf).unwrap();
                        }
                        writer.flush().unwrap();
                        index += stride;
                    }
                }
            }
        }
    }

    impl Widget for Rectangle {
        fn roundtrip<'d>(
            &'d mut self,
            _widget_x: u32,
            _widget_y: u32,
            dispatched: &Dispatch,
        ) -> Option<Damage> {
            if let Dispatch::Commit = dispatched {
                self.damaged = self.damaged == false;
            }
            None
        }
        fn damaged(&self) -> bool {
            self.damaged
        }
    }

    impl Rectangle {
        pub fn new(width: u32, height: u32, color: u32) -> Rectangle {
            Rectangle {
                color,
                width,
                height,
                damaged: true,
            }
        }
        pub fn empty(width: u32, height: u32) -> Rectangle {
            Rectangle {
                color: 0,
                width,
                height,
                damaged: true,
            }
        }
        pub fn square(size: u32, color: u32) -> Rectangle {
            Rectangle {
                color,
                width: size,
                height: size,
                damaged: true,
            }
        }
    }

    pub struct Border<W: Widget> {
        pub widget: W,
        color: u32,
        damaged: bool,
        size: (u32, u32, u32, u32),
    }

    impl<W: Widget> Geometry for Border<W> {
        fn get_width(&self) -> u32 {
            self.widget.get_width() + self.size.0 + self.size.2
        }
        fn get_height(&self) -> u32 {
            self.widget.get_height() + self.size.1 + self.size.3
        }
    }

    impl<W: Widget> Drawable for Border<W> {
        fn set_color(&mut self, color: u32) {
            self.color = color;
        }
        fn draw(&self, canvas: &mut Canvas, x: u32, y: u32) {
            if self.damaged {
                let bwidth = self.get_width();
                let bheight = self.get_height();

                Rectangle::new(bwidth, self.size.0, self.color).draw(canvas, x, y);
                Rectangle::new(bwidth, self.size.2, self.color).draw(canvas, x, y + bheight - self.size.2);
                Rectangle::new(self.size.1, bheight, self.color).draw(canvas, x + bwidth - self.size.1, y);
                Rectangle::new(self.size.3, bheight, self.color).draw(canvas, x, y);
            }
            self.widget.draw(canvas, x + self.size.0, y + self.size.3);
        }
    }

    impl<W: Widget> Widget for Border<W> {
        fn damaged(&self) -> bool {
            self.damaged
        }
        fn roundtrip<'d>(
            &'d mut self,
            widget_x: u32,
            widget_y: u32,
            dispatched: &Dispatch,
        ) -> Option<Damage> {
            if let Dispatch::Commit = dispatched {
                self.damaged = self.damaged == false;
            }
            self.widget
                .roundtrip(widget_x + self.size.0, widget_y + self.size.3, dispatched)
        }
    }

    impl<W: Widget> Border<W> {
        pub fn new(widget: W, size: u32, color: u32) -> Self {
            Self {
                widget,
                color,
                damaged: true,
                size: (size, size, size, size),
            }
        }
        pub fn set_border_size(&mut self, top: u32, right: u32, bottom: u32, left: u32) {
            self.size = (top, right, bottom, left);
        }
    }

    pub struct Background<W: Widget> {
        pub widget: W,
        damaged: bool,
        pub background: u32,
        padding: (u32, u32, u32, u32),
    }

    impl<W: Widget> Geometry for Background<W> {
        fn get_width(&self) -> u32 {
            self.widget.get_width() + self.padding.1 + self.padding.3
        }
        fn get_height(&self) -> u32 {
            self.widget.get_height() + self.padding.0 + self.padding.2
        }
    }

    impl<W: Widget> Drawable for Background<W> {
        fn set_color(&mut self, color: u32) {
            self.background = color;
        }
        fn draw(&self, canvas: &mut Canvas, x: u32, y: u32) {
            if self.damaged {
                Rectangle::new(self.get_width(), self.get_height(), self.background).draw(canvas, x, y);
            }
            self.widget
                .draw(canvas, x + self.padding.3, y + self.padding.0);
        }
    }

    impl<W: Widget> Widget for Background<W> {
        fn damaged(&self) -> bool {
            self.damaged
        }
        fn roundtrip<'d>(
            &'d mut self,
            widget_x: u32,
            widget_y: u32,
            dispatched: &Dispatch,
        ) -> Option<Damage> {
            if let Dispatch::Commit = dispatched {
                self.damaged = self.damaged == false;
            }
            self.widget.roundtrip(
                widget_x + self.padding.3,
                widget_y + self.padding.0,
                dispatched,
            )
        }
    }

    impl<W: Widget> Background<W> {
        pub fn new(widget: W, color: u32, padding: u32) -> Background<W> {
            Background {
                widget: widget,
                damaged: true,
                background: color,
                padding: (padding, padding, padding, padding),
            }
        }
        pub fn set_padding(&mut self, top: u32, right: u32, bottom: u32, left: u32) {
            self.padding = (top, right, bottom, left);
        }
    }
}
