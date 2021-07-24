use crate::*;
use crate::widgets::{Inner, Rectangle};

#[derive(Clone)]
pub struct Wbox {
    background: Rectangle,
    widgets: Vec<Inner>,
}

impl Container for Wbox {
    fn len(&self) -> u32 {
        self.widgets.len() as u32
    }
    fn add(&mut self, widget: impl Widget + 'static) -> Result<(), Error> {
        self.widgets.push(Inner::new(widget));
        Ok(())
    }
    fn put(&mut self, widget: Inner) -> Result<(), Error> {
        self.widgets.push(widget);
        Ok(())
    }
    fn get_child(&self) -> Result<&dyn Widget, Error> {
        Err(Error::Message("get_child is not valid on \"wbox\""))
    }
}

impl Widget for Wbox {}

impl Geometry for Wbox {
    fn get_width(&self) -> u32 {
        self.background.get_width()
    }
    fn get_height(&self) -> u32 {
        self.background.get_height()
    }
    fn contains<'d>(&'d mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage<'d> {
        for w in &mut self.widgets {
            let (rx, ry) = w.get_location();
            let ev = w.contains(widget_x + rx, widget_y + ry, x, y, event);
            if ev.is_some() {
                return ev
            }
        }
        Damage::None
    }
}

impl Drawable for Wbox {
    fn set_content(&mut self, content: Content) {
        self.background.set_content(content);
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.background.draw(canvas, width, x, y);
        for w in &self.widgets {
            let (x, y) = w.get_location();
            w.draw(canvas, width, x, y);
        }
    }
}

impl Wbox {
    pub fn new(background: Rectangle) -> Wbox {
        Wbox {
            background,
            widgets: Vec::new(),
        }
    }
    pub fn insert(&mut self, widget: impl Widget + 'static, x: u32, y: u32) {
        let inner = Inner::new_at(widget, x, y);
        self.put(inner).unwrap();
    }
    pub fn widgets(&self) -> &Vec<Inner> {
        &self.widgets
    }
}
