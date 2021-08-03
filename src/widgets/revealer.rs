use crate::widgets::*;

#[derive(Clone)]
pub struct Revealer<N: Widget, R: Widget> {
    state: bool,
    normal: N,
    reveal: R,
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
    fn contains<'d>(&'d mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        if self.state {
            self.reveal.contains(widget_x, widget_y, x, y, event)
        } else {
            self.normal.contains(widget_x, widget_y, x, y, event)
        }
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(),Error> {
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

impl<N: Widget, R: Widget> Widget for Revealer<N, R> {
    fn send_action<'s>(&'s mut self, action: Action, event_loop: &mut Vec<Damage>, widget_x: u32, widget_y: u32) {
        if self.state {
            self.reveal.send_action(action, event_loop, widget_x, widget_y);
        } else {
            self.normal.send_action(action, event_loop, widget_x, widget_y);
        }
    }
}
