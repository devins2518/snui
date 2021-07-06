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
    fn add(&mut self, _object: impl Drawable + 'static) -> Result<(), Error> {
        Err(Error::Overflow("listbox", 1))
    }
    fn get_location(&self) -> (u32, u32) {
        (self.x, self.y)
    }
    fn set_location(&mut self, x: u32, y: u32) {
        self.x = x;
        self.y = y;
    }
    fn put(&mut self, object: Inner) -> Result<(), Error> {
        Err(Error::Overflow("inner", 0))
    }
    /*
    fn get_child(&self) -> Vec<&Inner> {
        //vec![&*self.child]
        Vec::new()
    }
    */
}

impl Geometry for Inner {
    fn get_width(&self) -> u32 {
        self.child.get_width()
    }
    fn get_height(&self) -> u32 {
        self.child.get_height()
    }
    fn contains(&mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        if x > self.x
            && y > self.y
            && x < self.x + self.child.get_width()
            && y < self.y + self.child.get_height()
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
}
