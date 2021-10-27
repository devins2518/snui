use crate::widgets::primitives::WidgetShell;
use crate::*;

pub struct Revealer<N: Widget, R: Widget> {
    normal: N,
    reveal: R,
    state: bool,
}

impl<N: Widget, R: Widget> Drawable for Revealer<N, R> {
    fn set_color(&mut self, color: u32) {
        if self.state {
            self.reveal.set_color(color);
        } else {
            self.normal.set_color(color);
        }
    }
    fn draw(&self, ctx: &mut Context, x: f32, y: f32) {
        if self.state {
            self.reveal.draw(ctx, x, y)
        } else {
            self.normal.draw(ctx, x, y)
        }
    }
}

impl<N: Widget, R: Widget> Geometry for Revealer<N, R> {
    fn width(&self) -> f32 {
        if self.state {
            self.reveal.width()
        } else {
            self.normal.width()
        }
    }
    fn height(&self) -> f32 {
        if self.state {
            self.reveal.height()
        } else {
            self.normal.height()
        }
    }
}

impl<N: Widget, R: Widget> Revealer<N, R> {
    pub fn new(normal: N, reveal: R) -> WidgetShell<Revealer<N, R>> {
        WidgetShell::default(Revealer {
            state: false,
            normal,
            reveal,
        })
    }
    pub fn toggle(&mut self) {
        self.state = self.state == false;
    }
}

impl<N: Widget, R: Widget> Widget for Revealer<N, R> {
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, ctx: &mut Context, dispatch: &Dispatch) {
        if self.state {
            self.reveal.roundtrip(wx, wy, ctx, dispatch)
        } else {
            self.normal.roundtrip(wx, wy, ctx, dispatch)
        }
    }
}
