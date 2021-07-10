use crate::snui::*;
use crate::widgets::*;

pub struct Inner {
    x: u32,
    y: u32,
    child: Box<dyn Widget>,
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
    fn get_child(&self) -> Result<&dyn Widget,Error> {
        Ok(&*self.child)
    }
}

impl Geometry for Inner {
    fn get_width(&self) -> u32 {
        self.child.get_width()
    }
    fn get_height(&self) -> u32 {
        self.child.get_height()
    }
    fn contains(&mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        if x > widget_x
            && y > widget_y
            && x < widget_x + self.child.get_width()
            && y < widget_y + self.child.get_height()
        {
            self.child.contains(widget_x, widget_y, x, y, event)
        } else {
            Damage::None
        }
    }
}

impl Drawable for Inner {
    fn set_content(&mut self, content: Content) {
        self.child.set_content(content);
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
        self.child.draw(canvas, x, y);
    }
}

impl Widget for Inner {}

impl Inner {
    pub fn new(child: impl Widget + 'static) -> Inner {
        Inner {
            x: 0,
            y: 0,
            child: Box::new(child),
        }
    }
    pub fn new_at(child: impl Widget + 'static, x: u32, y: u32) -> Inner {
        Inner {
            x,
            y,
            child: Box::new(child),
        }
    }
    pub fn get_location(&self) -> (u32, u32) {
        (self.x, self.y)
    }
    pub fn set_location(&mut self, x: u32, y: u32) {
        self.x = x;
        self.y = y;
    }
}
