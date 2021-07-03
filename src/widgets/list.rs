use crate::snui::*;
use crate::widgets::{Surface, Rectangle};

pub struct List {
    content: Content,
    margin: u32,
    orientation: Orientation,
    capacity: Option<u32>,
    widgets: Vec<Box<Drawable>>,
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
        let (mut rx, mut ry) = (self.margin / 2, self.margin / 2);
        for w in &self.widgets {
            w.draw(canvas, rx, ry);
            match self.orientation {
                Orientation::Horizontal => {
                    rx += ((2 * w.get_width()) + self.margin) / 2;
                }
                Orientation::Vertical => {
                    ry += ((2 * w.get_height()) + self.margin) / 2;
                }
            }
        }
    }
    fn contains(&mut self, x: u32, y: u32, event: Input) -> bool {
        let (mut rx, mut ry) = (self.margin / 2, self.margin / 2);
        for w in &mut self.widgets {
            if x > rx && x < rx + w.get_width()
            && y > ry && y < ry + w.get_height() {
                w.contains(x, y, event);
            } {
                match self.orientation {
                    Orientation::Horizontal => {
                        rx += ((2 * w.get_width()) + self.margin) / 2;
                    }
                    Orientation::Vertical => {
                        ry += ((2 * w.get_height()) + self.margin) / 2;
                    }
                }
            }
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
        if let Some(capacity) = self.capacity {
            if self.len() == capacity {
                Err(Error::Overflow("wbox", capacity as u32))
            } else {
                self.widgets.push(Box::new(object));
                Ok(())
            }
        } else {
            self.widgets.push(Box::new(object));
            Ok(())
        }
    }
    // Returns the list of child windows
    fn get_child(&self) -> Vec<&Drawable> {
        let mut v = Vec::new();
        for w in &self.widgets {
            v.push(&**w);
        }
        v
    }
}

impl List {
    pub fn new(orientation: Orientation, capacity: Option<u32>) -> Self {
        let widgets = match capacity {
            Some(capacity) => Vec::with_capacity(capacity as usize),
            None => Vec::new(),
        };
        List {
            content: Content::Empty,
            capacity,
            widgets,
            margin: 0,
            orientation,
        }
    }
    pub fn set_margin(&mut self, margin: u32) {
        self.margin = margin;
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
