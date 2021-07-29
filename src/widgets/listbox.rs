use crate::*;
use crate::widgets::Inner;

#[derive(Clone)]
pub struct ListBox {
    spacing: u32,
    widgets: Vec<Inner>,
    orientation: Orientation,
}

impl Geometry for ListBox {
    fn get_width(&self) -> u32 {
        if let Some(widget) = self.widgets.first() {
            let (x, _y) = widget.get_location();
            let mut xf = x;
            let mut width = widget.get_width();
            for w in self.widgets() {
                let (lx, _ly) = w.get_location();
                if lx > xf {
                    xf = lx;
                    width = w.get_width();
                }
            }
            return xf - x + width;
        } else { 0 }
    }
    fn get_height(&self) -> u32 {
        if let Some(widget) = self.widgets.first() {
            let (_x, y) = widget.get_location();
            let mut yf = y;
            let mut height = widget.get_height();
            for w in self.widgets() {
                let (_lx, ly) = w.get_location();
                if ly > yf {
                    yf = ly;
                    height = w.get_height();
                }
            }
            return yf - y + height;
        } else { 0 }
    }
    fn contains<'d>(&'d mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage<'d> {
        for l in &mut self.widgets {
            let (dx, dy) = l.get_location();
            let ev = l.contains(widget_x + dx, widget_y + dy, x, y, event);
            if ev.is_some() {
                return ev
            }
        }
        Damage::None
    }
}

impl Widget for ListBox {}

impl Drawable for ListBox {
    fn set_content(&mut self, _content: Content) {
    	eprintln!("Attempted to perform illegal operation on Listbox!");
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        for w in &self.widgets {
            let (dx, dy) = w.get_location();
            w.draw(canvas, width, x+dx, y+dy);
        }
    }
}

impl Container for ListBox {
    fn len(&self) -> u32 {
        self.widgets.len() as u32
    }
    fn add(&mut self, widget: impl Widget + 'static) -> Result<(), Error> {
        let last_element = self.widgets.last();
        let (x, y) = if let Some(w) = last_element {
            let (mut x, mut y) = w.get_location();
            match self.orientation {
                Orientation::Horizontal => {
                    x += w.get_width() + self.spacing;
                }
                Orientation::Vertical => {
                    y += w.get_height() + self.spacing;
                }
            }
            (x, y)
        } else {
            (0, 0)
        };
        self.widgets.push(Inner::new_at(widget, x, y));
        Ok(())
    }
    fn put(&mut self, _widget: Inner) -> Result<(), Error> {
        Err(Error::Message("widgets cannot be put in \"listbox\""))
    }
    fn get_child(&self) -> Result<&dyn Widget, Error> {
        Err(Error::Message("get_child is not valid on \"listbox\""))
    }
}

impl ListBox {
    pub fn new(orientation: Orientation) -> Self {
        ListBox {
            widgets: Vec::new(),
            spacing: 0,
            orientation,
        }
    }
    pub fn set_spacing(&mut self, spacing: u32) {
        self.spacing = spacing;
    }
    pub fn widgets(&self) -> &Vec<Inner> {
        &self.widgets
    }
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
}
