use crate::snui::*;
use crate::widgets::*;

pub struct ListBox {
    x: u32,
    y: u32,
    child: Box<Drawable>,
}

impl Container for ListBox {
    fn len(&self) -> u32 {
        1
    }
    fn add(&mut self, object: impl Drawable + 'static) -> Result<(), Error> {
        Err(Error::Overflow("listbox", 1))
    }
    fn get_child(&self) -> Vec<&Drawable> {
        vec![&*self.child]
    }
}

impl Drawable for ListBox {
    fn set_content(&mut self, content: Content) {
        self.child.set_content(content);
    }
    fn contains(&mut self, x: u32, y: u32, event: Input) -> bool {
        if x > self.x
            && x < self.x + self.child.get_width()
            && y > self.y
            && y < self.y + self.child.get_height()
        {
            self.child.contains(x, y, event)
        } else {
            false
        }
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
        self.child.draw(canvas, x, y);
    }
    fn get_width(&self) -> u32 {
        self.child.get_width()
    }
    fn get_height(&self) -> u32 {
        self.child.get_height()
    }
}

impl ListBox {
    pub fn new(child: impl Drawable + 'static) -> ListBox {
        ListBox {
            x: 0,
            y: 0,
            child: Box::new(child),
        }
    }
    pub fn new_at(child: impl Drawable + 'static, x: u32, y: u32) -> ListBox {
        ListBox {
            x,
            y,
            child: Box::new(child),
        }
    }
    pub fn set_location(&mut self, x: u32, y: u32) {
        self.x = x;
        self.y = y;
    }
    pub fn get_location(&self) -> (u32, u32) {
        (self.x, self.y)
    }
}
