pub mod container;
pub mod primitives;
pub mod text;

use crate::*;
use raqote::*;
pub use text::Label;
use std::ops::{Deref, DerefMut};
pub use container::{layout::WidgetLayout, Wbox};

pub fn u32_to_source(color: u32) -> SolidSource {
    let color = color.to_be_bytes();
    SolidSource {
        a: color[0],
        r: color[1],
        g: color[2],
        b: color[3],
    }
}

pub fn blend(pix_a: &[u8], pix_b: &[u8], t: f32) -> [u8; 4] {
    let (r_a, g_a, b_a, a_a) = (
        pix_a[1] as f32,
        pix_a[2] as f32,
        pix_a[3] as f32,
        pix_a[0] as f32,
    );
    let (r_b, g_b, b_b, a_b) = (
        pix_b[1] as f32,
        pix_b[2] as f32,
        pix_b[3] as f32,
        pix_b[0] as f32,
    );
    let red = blend_f32(r_a, r_b, t);
    let green = blend_f32(g_a, g_b, t);
    let blue = blend_f32(b_a, b_b, t);
    let alpha = blend_f32(a_a, a_b, t);
    [alpha as u8, red as u8, green as u8, blue as u8]
}

fn blend_f32(a: f32, b: f32, r: f32) -> f32 {
    a + ((b - a) * r)
}

pub struct Button<W: Geometry + Drawable> {
    widget: W,
    focused: bool,
    cb: Box<dyn for<'d> FnMut(&'d mut W, Pointer) + 'static>,
}

impl<W: Widget> Button<W> {
    pub fn new(
        widget: W,
        cb: impl for<'d> FnMut(&'d mut W, Pointer) + 'static,
    ) -> Self {
        Self {
            widget,
            focused: false,
            cb: Box::new(cb),
        }
    }
}

impl<W: Widget> Geometry for Button<W> {
    fn width(&self) -> f32 {
        self.widget.width()
    }
    fn height(&self) -> f32 {
        self.widget.height()
    }
}

impl<W: Widget> Drawable for Button<W> {
    fn set_color(&mut self, color: u32) {
        self.widget.set_color(color);
    }
    fn draw(&self, ctx: &mut Context, x: f32, y: f32) {
        self.widget.draw(ctx, x, y)
    }
}

impl<W: Widget> Widget for Button<W> {
    fn create_node(&self, x: f32, y: f32) -> RenderNode {
        self.widget.create_node(x, y)
    }
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, ctx: &mut Context, dispatch: &Dispatch) {
        match dispatch {
            Dispatch::Pointer(x, y, pointer) => match pointer {
                Pointer::Leave => {
                    if self.focused {
                        self.focused = false;
                        (self.cb)(&mut self.widget, *pointer)
                    }
                }
                _ => {
                    if *x > wx && *y > wy && *x < wx + self.width() && *y < wy + self.height() {
                        if self.focused {
                            (self.cb)(&mut self.widget, *pointer)
                        } else {
                            self.focused = true;
                            (self.cb)(&mut self.widget, Pointer::Enter)
                        }
                    } else if self.focused {
                        self.focused = false;
                        (self.cb)(&mut self.widget, Pointer::Leave)
                    }
                }
            },
            _ => {}
        }
        self.widget.roundtrip(wx, wy, ctx, dispatch);
    }
}

impl<W: Widget> Deref for Button<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<W: Widget> DerefMut for Button<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}
