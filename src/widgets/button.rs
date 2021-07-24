use crate::widgets::*;
use std::rc::Rc;

#[derive(Clone)]
pub struct Button<W: Widget> {
    widget: W,
    callback: Rc<dyn FnMut(&mut W, u32, u32, Input) -> Damage>,
}

impl<W: Widget> Geometry for Button<W> {
    fn get_width(&self) -> u32 {
        self.widget.get_width()
    }
    fn get_height(&self) -> u32 {
        self.widget.get_height()
    }
    fn contains(&mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        if x > widget_x
            && y > widget_y
            && x < widget_x + self.widget.get_width()
            && y < widget_y + self.widget.get_height()
        {
            Rc::get_mut(&mut self.callback).unwrap()(&mut self.widget, widget_x, widget_y, event)
        } else {
            Damage::None
        }
    }
}

impl<W: Widget> Drawable for Button<W> {
    fn set_content(&mut self, content: Content) {
        self.widget.set_content(content);
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.widget.draw(canvas, width, x, y)
    }
}

impl<W: Widget> Widget for Button<W> {}

impl<W: Widget> Button<W> {
    pub fn new(widget: W, f: impl FnMut(&mut W, u32, u32, Input) -> Damage + 'static) -> Button<W> {
        Button {
            widget,
            callback: Rc::new(f),
        }
    }
}
