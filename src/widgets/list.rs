use crate::snui::*;
use crate::widgets::{ListBox, Surface, Rectangle};

pub struct List {
    content: Content,
    margin: u32,
    orientation: Orientation,
    capacity: Option<u32>,
    widgets: Vec<ListBox>,
}

impl Drawable for List {
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
                width + self.margin - self.margin%2
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
                height + self.margin - self.margin%2
            }
        }
    }
    fn set_content(&mut self, content: Content) {
        self.content = content;
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
        let mut bg = Rectangle::new(
            self.get_width(),
            self.get_height(),
        );
        bg.set_content(self.content);
        bg.draw(canvas, x, y);
        for w in &self.widgets {
            let (x, y) = w.get_location();
            w.draw(canvas, x, y);
        }
    }
    fn contains(&mut self, x: u32, y: u32, event: Input) -> bool {
        let (mut rx, mut ry) = (self.margin / 2, self.margin / 2);
        for l in &mut self.widgets {
            l.contains(x, y, event);
        }
        false
    }
}

impl Container for List {
    fn len(&self) -> u32 {
        self.widgets.len() as u32
    }
    // Appends an object at the end of a Container
    fn add(&mut self, object: impl Drawable + 'static) -> Result<(), Error> {
        let mut list_element = ListBox::new(object);
        let last_element = self.widgets.last();
        if let Some(w) = last_element {
            let (mut x, mut y) = w.get_location();
            match self.orientation {
                Orientation::Horizontal => {
                    x += ((2 * w.get_width()) + self.margin) / 2;
                }
                Orientation::Vertical => {
                    y += ((2 * w.get_height()) + self.margin) / 2;
                }
            }
            list_element.set_location(x, y);
        } else {
            list_element.set_location(self.margin / 2, self.margin / 2);
        }
        self.widgets.push(list_element);
        Ok(())
    }
    // Returns the list of child windows
    fn get_child(&self) -> Vec<&Drawable> {
        let mut v = Vec::new();
        for w in &self.widgets {
            v.append(&mut w.get_child())
        }
        v
    }
}

impl List {
    pub fn new(orientation: Orientation, capacity: Option<u32>) -> Self {
        List {
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
    pub fn set(&mut self, object: impl Drawable + 'static, index: u32) -> Result<(), Error> {
        if index > self.len() {
            Err(Error::Overflow("list", self.len()))
        } else {
            let (x, y) = self.widgets[index as usize].get_location();
            let list_element = ListBox::new(object);
            self.widgets[index as usize] = list_element;
            Ok(())
        }
    }
    pub fn get_listbox(&self, index: u32) -> Result<&ListBox, Error> {
        if index < self.len() {
            Ok(&self.widgets[index as usize])
        } else {
            Err(Error::Overflow("list", self.len()))
        }
    }
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
    pub fn to_surface(&self) -> Surface {
        let mut surface = Surface::new(
            self.get_width() + self.margin,
            self.get_height() + self.margin,
            Content::Empty,
        );
        self.draw(&mut surface, 0, 0);
        surface
    }
}
