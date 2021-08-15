use crate::widgets::*;
use active::command::Command;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub enum Event {
    MouseClick {
        time: u32,
        button: u32,
        pressed: bool,
    },
    Enter,
    Leave,
}

pub struct Button<W: Widget> {
    widget: W,
    callback: Rc<dyn Fn(&mut W, u32, u32, u32, u32, Event) -> Damage>,
}

impl<W: Widget> Geometry for Button<W> {
    fn get_width(&self) -> u32 {
        self.widget.get_width()
    }
    fn get_height(&self) -> u32 {
        self.widget.get_height()
    }
    fn contains<'d>(
        &'d mut self,
        widget_x: u32,
        widget_y: u32,
        x: u32,
        y: u32,
        event: Event,
    ) -> Damage {
        if x > widget_x
            && y > widget_y
            && x < widget_x + self.get_width()
            && y < widget_y + self.get_height()
        {
            self.callback.deref()(&mut self.widget, widget_x, widget_y, x, y, event)
        } else {
            Damage::None
        }
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.widget.resize(width, height)
    }
}

impl<W: Widget> Drawable for Button<W> {
    fn set_color(&mut self, color: u32) {
        self.widget.set_color(color);
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.widget.draw(canvas, width, x, y)
    }
}

impl<W: Widget> Widget for Button<W> {
    fn send_command<'s>(
        &'s mut self,
        command: Command,
        damage_queue: &mut Vec<Damage<'s>>,
        x: u32,
        y: u32,
    ) {
        self.widget.send_command(command, damage_queue, x, y)
    }
}

impl<W: Widget> Button<W> {
    pub fn new(
        widget: W,
        f: impl Fn(&mut W, u32, u32, u32, u32, Event) -> Damage + 'static,
    ) -> Button<W> {
        Button {
            widget: widget,
            callback: Rc::new(f),
        }
    }
}
}

pub mod command {
use crate::widgets::*;
use active::pointer;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub enum Command<'a> {
    Name(&'a str),
    Key(&'a str, Key),
    Hide,
    Destroy,
    Data(&'a str, &'a dyn std::any::Any),
}

impl<'a> Command<'a> {
    pub fn eq(&self, value: &'a str) -> bool {
        match &self {
            Command::Name(name) => name.eq(&value),
            Command::Key(name, _) => name.eq(&value),
            Command::Data(name, _) => name.eq(&value),
            _ => false,
        }
    }
    pub fn get<T: std::any::Any>(&self) -> Option<&T> {
        match self {
            Command::Data(_, value) => value.downcast_ref(),
            _ => None,
        }
    }
}

pub struct Actionnable<W: Widget> {
    pub widget: W,
    callback: Rc<dyn for<'a, 'd> Fn(&'d mut W, Command) -> bool>,
}

impl<W: Widget> Geometry for Actionnable<W> {
    fn get_width(&self) -> u32 {
        self.widget.get_width()
    }
    fn get_height(&self) -> u32 {
        self.widget.get_height()
    }
    fn contains<'d>(
        &'d mut self,
        widget_x: u32,
        widget_y: u32,
        x: u32,
        y: u32,
        event: pointer::Event,
    ) -> Damage {
        self.widget.contains(widget_x, widget_y, x, y, event)
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.widget.resize(width, height)
    }
}

impl<W: Widget> Drawable for Actionnable<W> {
    fn set_color(&mut self, color: u32) {
        self.widget.set_color(color);
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.widget.draw(canvas, width, x, y)
    }
}

impl<W: Widget> Widget for Actionnable<W> {
    fn send_command<'s>(
        &'s mut self,
        command: Command,
        damage_queue: &mut Vec<Damage<'s>>,
        x: u32,
        y: u32,
    ) {
        if self.callback.deref()(&mut self.widget, command) {
            damage_queue.push(Damage::new(&self.widget, x, y));
        }
    }
}

impl<W: Widget> Actionnable<W> {
    pub fn new(widget: W, f: impl for<'a> Fn(&mut W, Command) -> bool + 'static) -> Self {
        Self {
            widget: widget,
            callback: Rc::new(f),
        }
    }
}
