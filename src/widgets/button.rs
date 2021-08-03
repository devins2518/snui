use std::rc::Rc;
use crate::widgets::*;
use crate::Action;
use std::ops::Deref;

#[derive(Clone)]
pub struct Button<W: Widget + Clone> {
    widget: Rc<W>,
    callback: Rc<dyn Fn(&mut W, u32, u32, u32, u32, Input) -> Damage>,
}

impl<W: Widget + Clone> Geometry for Button<W> {
    fn get_width(&self) -> u32 {
        self.widget.as_ref().get_width()
    }
    fn get_height(&self) -> u32 {
        self.widget.as_ref().get_height()
    }
    fn contains<'d>(&'d mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        if x > widget_x
            && y > widget_y
            && x < widget_x + self.get_width()
            && y < widget_y + self.get_height()
        {
            self.callback.deref()(Rc::get_mut(&mut self.widget).unwrap(), widget_x, widget_y, x, y, event)
        } else {
            Damage::None
        }
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(),Error> {
        Rc::get_mut(&mut self.widget).unwrap().resize(width, height)
    }
}

impl<W: Widget + Clone> Drawable for Button<W> {
    fn set_color(&mut self, color: u32) {
        Rc::get_mut(&mut self.widget).unwrap().set_color(color);
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.widget.as_ref().draw(canvas, width, x, y)
    }
}

impl<W: Widget + Clone> Widget for Button<W> {
    fn action<'s>(&'s mut self, name: Action, event_loop: &mut Vec<Damage>, widget_x: u32, widget_y: u32) {
        Rc::get_mut(&mut self.widget).unwrap().action(name, event_loop, widget_x, widget_y);
    }
}

impl<W: Widget + Clone> Button<W> {
    pub fn new(widget: W, f: impl Fn(&mut W, u32, u32, u32, u32, Input) -> Damage + 'static) -> Button<W> {
        Button {
            widget: Rc::new(widget),
            callback: Rc::new(f),
        }
    }
}

#[derive(Clone)]
pub struct Actionnable<W: Widget + Clone> {
    name: String,
    widget: Rc<W>,
    callback: Rc<dyn Fn(&mut W, &mut Vec<Damage>, u32, u32)>,
}

impl<W: Widget + Clone> Geometry for Actionnable<W> {
    fn get_width(&self) -> u32 {
        self.widget.as_ref().get_width()
    }
    fn get_height(&self) -> u32 {
        self.widget.as_ref().get_height()
    }
    fn contains<'d>(&'d mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        Rc::get_mut(&mut self.widget).unwrap().contains(widget_x, widget_y, x, y, event)
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(),Error> {
        Rc::get_mut(&mut self.widget).unwrap().resize(width, height)
    }
}

impl<W: Widget + Clone> Drawable for Actionnable<W> {
    fn set_color(&mut self, color: u32) {
        Rc::get_mut(&mut self.widget).unwrap().set_color(color);
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.widget.as_ref().draw(canvas, width, x, y)
    }
}

impl<W: Widget + Clone> Widget for Actionnable<W> {
    fn action<'s>(&'s mut self, name: Action, event_loop: &mut Vec<Damage>, widget_x: u32, widget_y: u32) {
        if name.eq(&self.name) {
            self.callback.deref()(Rc::get_mut(&mut self.widget).unwrap(), event_loop, widget_x, widget_y);
        }
    }
}

impl<W: Widget + Clone> Actionnable<W> {
    pub fn new<'s>(name: &'s str, widget: W, f: impl Fn(&mut W, &mut Vec<Damage>, u32, u32) + 'static) -> Self {
        Self {
            name: name.to_owned(),
            widget: Rc::new(widget),
            callback: Rc::new(f),
        }
    }
}
