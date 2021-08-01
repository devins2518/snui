use crate::*;
use std::rc::Rc;

#[derive(Clone)]
pub struct Inner {
    x: u32,
    y: u32,
    child: Rc<dyn Widget>,
}

impl Geometry for Inner {
    fn get_width(&self) -> u32 {
        self.child.as_ref().get_width()
    }
    fn get_height(&self) -> u32 {
        self.child.as_ref().get_height()
    }
    fn contains<'d>(&'d mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage<'d> {
        if x > widget_x
            && y > widget_y
            && x < widget_x + self.get_width()
            && y < widget_y + self.get_height()
        {
            Rc::get_mut(&mut self.child).unwrap().contains(widget_x, widget_y, x, y, event)
        } else {
            Damage::None
        }
    }
}

impl Container for Inner {
    fn len(&self) -> u32 {
        1
    }
    fn add(&mut self, _widget: impl Drawable + 'static) -> Result<(), Error> {
        Err(Error::Overflow("inner", 1))
    }
    fn put(&mut self, _widget: Inner) -> Result<(), Error> {
        Err(Error::Overflow("inner", 1))
    }
    fn get_child(&self) -> Result<&dyn Widget, Error> {
        Ok(&*self.child)
    }
}

impl Drawable for Inner {
    fn set_color(&mut self, color: u32) {
        Rc::get_mut(&mut self.child).unwrap().set_color(color)
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.child.draw(canvas, width, x, y);
    }
}

impl Widget for Inner {}

impl Inner {
    pub fn new(child: impl Widget + 'static) -> Inner {
        Inner {
            x: 0,
            y: 0,
            child: Rc::new(child),
        }
    }
    pub fn new_at(child: impl Widget + 'static, x: u32, y: u32) -> Inner {
        Inner {
            x,
            y,
            child: Rc::new(child),
        }
    }
    pub fn get_location(&self) -> (u32, u32) {
        (self.x, self.y)
    }
    pub fn set_location(&mut self, x: u32, y: u32) {
        self.x = x;
        self.y = y;
    }
    pub fn translate(&mut self, x: u32, y: u32) {
        self.x += x;
        self.y += y;
    }
}
