use crate::*;
use std::rc::Rc;
use crate::widgets::{Inner, Rectangle};

#[derive(Clone)]
pub enum Alignment {
    Start,
    Center,
    End,
}

#[derive(Clone)]
pub struct Wbox {
    spacing: u32,
    pub widgets: Vec<Inner>,
    background: Option<Rc<dyn Widget>>,
    orientation: Orientation,
}

impl Geometry for Wbox {
    fn get_width(&self) -> u32 {
        if let Some(bg) = &self.background {
            bg.get_width()
        } else {
            if let Some(widget) = self.widgets.first() {
                let (x, _y) = widget.get_location();
                let mut xf = x + widget.get_width();
                for w in &self.widgets {
                    let (mut lx, _ly) = w.get_location();
                    lx += w.get_width();
                    if lx > xf {
                        xf = lx;
                    }
                }
                return xf;
            } else { 0 }
        }
    }
    fn get_height(&self) -> u32 {
        if let Some(bg) = &self.background {
            bg.get_height()
        } else {
            if let Some(widget) = self.widgets.first() {
                let (_x, y) = widget.get_location();
                let mut yf = y + widget.get_height();
                for w in &self.widgets {
                    let (_lx, mut ly) = w.get_location();
                    ly += w.get_height();
                    if ly > yf {
                        yf = ly;
                    }
                }
                return yf;
            } else { 0 }
        }
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

impl Widget for Wbox {}

impl Drawable for Wbox {
    fn set_color(&mut self, color: u32) {
        if let Some(bg) = &mut self.background {
            Rc::get_mut(bg).unwrap().set_color(color);
        }
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        if let Some(bg) = &self.background {
            bg.draw(canvas, width, x, y);
        }
        for w in &self.widgets {
            let (dx, dy) = w.get_location();
            w.draw(canvas, width, x+dx, y+dy);
        }
    }
}

impl Container for Wbox {
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
        if let Some(bg) = &self.background {
            if x + widget.get_width() <= bg.get_width()
            && y + widget.get_height() <= bg.get_height() {
                self.widgets.push(Inner::new_at(widget, x, y));
                Ok(())
            } else {
                Err(Error::Dimension("wbox", widget.get_width(), widget.get_height()))
            }
        } else {
            self.widgets.push(Inner::new_at(widget, x, y));
            Ok(())
        }
    }
    fn put(&mut self, widget: Inner) -> Result<(), Error> {
        self.widgets.push(widget);
        Ok(())
    }
    fn get_child(&self) -> Result<&dyn Widget, Error> {
        Err(Error::Message("get_child is not valid on \"wbox\""))
    }
}

impl Wbox {
    pub fn new() -> Self {
        Wbox {
            spacing: 0,
            background: None,
            widgets: Vec::new(),
            orientation: Orientation::Horizontal,
        }
    }
    pub fn new_with_spacing(spacing: u32) -> Self {
        Wbox {
            spacing,
            background: None,
            widgets: Vec::new(),
            orientation: Orientation::Horizontal,
        }
    }
    pub fn from(background: impl Widget + 'static) -> Self {
        Wbox {
            spacing: 0,
            background: Some(Rc::new(background)),
            widgets: Vec::new(),
            orientation: Orientation::Horizontal,
        }
    }
    pub fn new_with_size(w: u32, h: u32) -> Self {
        Wbox {
            spacing: 0,
            background: Some(Rc::new(Rectangle::empty(w, h))),
            widgets: Vec::new(),
            orientation: Orientation::Horizontal,
        }
    }
    pub fn new_orientation(orientation: Orientation) -> Self {
        Wbox {
            spacing: 0,
            background: None,
            widgets: Vec::new(),
            orientation,
        }
    }
    pub fn set_spacing(&mut self, spacing: u32) {
        self.spacing = spacing;
    }
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
}
