use crate::snui::*;
use crate::widgets::{Inner, Rectangle, Surface};

pub struct ListBox {
    background: Content,
    margin: u32,
    orientation: Orientation,
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
        self.background = content;
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
        let mut bg = Rectangle::new(self.get_width(), self.get_height());
        bg.set_content(self.background);
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
    // Appends an widget at the end of a Container
    fn add(&mut self, widget: impl Widget + 'static) -> Result<(), Error> {
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
        self.widgets.push(Inner::new_at(widget, x, y));
        Ok(())
    }
    fn put(&mut self, widget: Inner) -> Result<(), Error> {
        Err(Error::Message("widgets cannot be put in \"listbox\""))
    }
    fn get_child(&self) -> Result<&dyn Widget,Error> {
        Err(Error::Message("get_child is not valid on \"listbox\""))
    }
}

impl ListBox {
    pub fn new(orientation: Orientation) -> Self {
        ListBox {
            background: Content::Empty,
            widgets: Vec::new(),
            margin: 0,
            orientation,
        }
    }
    pub fn set_margin(&mut self, margin: u32) {
        self.margin = margin;
    }
    pub fn widgets(&self) -> &Vec<Inner> {
        &self.widgets
    }
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
}
