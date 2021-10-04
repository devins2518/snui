use crate::*;

#[derive(Clone)]
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
    fn draw(&self, canvas: &mut Canvas, x: u32, y: u32) {
        if self.state {
            self.reveal.draw(canvas, x, y)
        } else {
            self.normal.draw(canvas, x, y)
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
}

impl<N: Widget, R: Widget> Revealer<N, R> {
    pub fn new(normal: N, reveal: R) -> Revealer<N, R> {
        Revealer {
            state: false,
            normal,
            reveal,
        }
    }
    pub fn toggle(&mut self) {
        self.state = self.state == false;
    }
}

impl<N: Widget, R: Widget> Widget for Revealer<N, R> {
    fn damaged(&self) -> bool {
        if self.state {
            self.reveal.damaged()
        } else {
            self.normal.damaged()
        }
    }
    fn roundtrip<'d>(
        &'d mut self,
        widget_x: u32,
        widget_y: u32,
        dispatched: &Dispatch,
    ) -> Option<Damage> {
        if let Dispatch::Commit = dispatched {
            self.toggle();
            None
        } else {
            if self.state {
                self.reveal.roundtrip(widget_x, widget_y, dispatched)
            } else {
                self.normal.roundtrip(widget_x, widget_y, dispatched)
            }
        }
    }
}
