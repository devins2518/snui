use crate::snui::*;
use crate::widgets::{Inner, Rectangle, Surface};

pub struct ListBox {
    content: Content,
    margin: u32,
    orientation: Orientation,
    capacity: Option<u32>,
    widgets: Vec<Inner>,
}

impl Geometry for ListBox {
    fn get_width(&self) -> u32 {
        match self.orientation {
            Orientation::Horizontal => {
                let mut width = 0;
                width += (self.margin / 2) * (self.len() + 1);
                for w in &self.widgets {
                    width += w.get_width();
                }
                width as u32
            }
            _ => {
                let mut width = 0;
                for w in &self.widgets {
                    if w.get_width() > width {
                        width = w.get_width() as u32;
                    }
                }
                width + self.margin - self.margin % 2
            }
        }
    }
    fn get_height(&self) -> u32 {
        match self.orientation {
            Orientation::Vertical => {
                let mut height = 0;
                height += (self.margin / 2) * (self.len() + 1);
                for w in &self.widgets {
                    height += w.get_height();
                }
                height as u32
            }
            _ => {
                let mut height = 0;
                for w in &self.widgets {
                    if w.get_height() > height {
                        height = w.get_height() as u32;
                    }
                }
                height + self.margin - self.margin % 2
            }
        }
    }
    fn contains(&mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        for l in &mut self.widgets {
            let (rx, ry) = l.get_location();
            let msg = l.contains(widget_x + rx, widget_y + ry, x, y, event);
            match &msg {
                Damage::None => {}
                _ => return msg,
            }
        }
        Damage::None
    }
}

impl Widget for ListBox {}

impl Drawable for ListBox {
    fn set_content(&mut self, content: Content) {
        self.content = content;
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
        let mut bg = Rectangle::new(self.get_width(), self.get_height());
        bg.set_content(self.content);
        bg.draw(canvas, x, y);
        for w in &self.widgets {
            let (x, y) = w.get_location();
            w.draw(canvas, x, y);
        }
    }
}

impl Container for ListBox {
    fn len(&self) -> u32 {
        self.widgets.len() as u32
    }
    // Appends an object at the end of a Container
    fn add(&mut self, mut object: impl Widget + 'static) -> Result<(), Error> {
        let last_element = self.widgets.last();
        let (x, y) = if let Some(w) = last_element {
            let (mut x, mut y) = w.get_location();
            match self.orientation {
                Orientation::Horizontal => {
                    x += ((2 * w.get_width()) + self.margin) / 2;
                }
                Orientation::Vertical => {
                    y += ((2 * w.get_height()) + self.margin) / 2;
                }
            }
            (x, y)
        } else {
            (self.margin / 2, self.margin / 2)
        };
        self.widgets.push(Inner::new_at(object, x, y));
        Ok(())
    }
    fn get_location(&self) -> (u32, u32) {
        if self.len() > 0 {
            self.widgets[0].get_location()
        } else {
            (0, 0)
        }
    }
    fn set_location(&mut self, x: u32, y: u32) {
        if self.len() > 0 {
            self.widgets[0].set_location(x, y);
        }
    }
    fn put(&mut self, object: Inner) -> Result<(), Error> {
        self.widgets.push(object);
        Ok(())
    }
    /*
    // Returns the list of child windows
    fn get_child(&self) -> Vec<&Inner> {
        let mut v = Vec::new();
        for w in &self.widgets {
            v.append(&mut w.get_child())
        }
        v
    }
    */
}

impl ListBox {
    pub fn new(orientation: Orientation, capacity: Option<u32>) -> Self {
        ListBox {
            content: Content::Empty,
            capacity,
            widgets: Vec::new(),
            margin: 0,
            orientation,
        }
    }
    pub fn set_margin(&mut self, margin: u32) {
        self.margin = margin;
    }
    /*
    pub fn get_listbox(&self, index: u32) -> Result<&Inner, Error> {
        if index < self.len() {
            Ok(&self.widgets[index as usize])
        } else {
            Err(Error::Overflow("list", self.len()))
        }
    }
    */
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
}
