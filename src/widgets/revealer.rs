use crate::widgets::*;

pub struct Revealer<N: Widget, R: Widget> {
    state: bool,
    normal: N,
    reveal: R,
}

impl<N: Widget, R: Widget> Drawable for Revealer<N, R> {
    fn set_content(&mut self, content: Content) {
        if self.state {
            self.reveal.set_content(content);
        } else {
            self.normal.set_content(content);
        }
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
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
    fn contains(&mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        if self.state {
            self.reveal.contains(widget_x, widget_y, x, y, event)
        } else {
            self.normal.contains(widget_x, widget_y, x, y, event)
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
        if self.state {
            self.state = false
        } else {
            self.state = true
        }
    }
}
