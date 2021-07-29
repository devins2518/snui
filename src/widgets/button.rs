use std::rc::Rc;
use crate::widgets::*;
use std::ops::Deref;

#[derive(Clone)]
pub struct Button<W: Widget + Clone> {
    widget: W,
    callback: Rc<dyn Fn(&mut W, u32, u32, Input) -> Damage>,
}

impl<W: Widget + Clone> Geometry for Button<W> {
    fn get_width(&self) -> u32 {
        self.widget.get_width()
    }
    fn get_height(&self) -> u32 {
        self.widget.get_height()
    }
    fn contains<'d>(&'d mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage<'d> {
        if x > widget_x
            && y > widget_y
            && x < widget_x + self.get_width()
            && y < widget_y + self.get_height()
        {
            self.callback.deref()(&mut self.widget, x, y, event)
        } else {
            Damage::None
        }
    }
}

impl<W: Widget + Clone> Drawable for Button<W> {
    fn set_content(&mut self, content: Content) {
        self.widget.set_content(content);
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.widget.draw(canvas, width, x, y)
    }
}

impl<W: Widget + Clone> Widget for Button<W> {}

impl<W: Widget + Clone> Button<W> {
    pub fn new(widget: W, f: impl Fn(&mut W, u32, u32, Input) -> Damage + 'static) -> Button<W> {
        Button {
            widget: widget,
            callback: Rc::new(f),
        }
    }
}
