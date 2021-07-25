use std::rc::Rc;
use crate::State;
use crate::widgets::*;

#[derive(Clone)]
pub struct Button<W: Widget + Clone> {
    widget: W,
    callback: Rc<dyn FnMut(&mut W, Input) -> State>,
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
            && x < widget_x + self.widget.get_width()
            && y < widget_y + self.widget.get_height()
        {
            match Rc::get_mut(&mut self.callback).unwrap()(&mut self.widget, event) {
                State::Hide => Damage::Hide,
                State::Close => Damage::Destroy,
                State::Show => Damage::new(&self.widget, widget_x, widget_y),
                State::Same => Damage::None,
            }
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
    pub fn new(widget: W, f: impl FnMut(&mut W, Input) -> State + 'static) -> Button<W> {
        Button {
            widget,
            callback: Rc::new(f),
        }
    }
}
