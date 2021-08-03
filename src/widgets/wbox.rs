use crate::*;
use std::rc::Rc;
use crate::widgets::anchor;
use crate::widgets::{Inner, Rectangle};

#[derive(Copy, Clone)]
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
    alignment: Alignment,
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
    fn resize(&mut self, width: u32, height: u32) -> Result<(),Error> {
        if let Some(bg) = self.background.as_mut() {
            Rc::get_mut(bg).unwrap().resize(width, height)
        } else {
            Ok(())
        }
    }
}

impl Drawable for Wbox {
    fn set_color(&mut self, color: u32) {
        if let Some(bg) = &mut self.background {
            Rc::get_mut(bg).unwrap().set_color(color);
        }
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        let sized;
        if let Some(bg) = &self.background {
            sized = true;
            bg.draw(canvas, width, x, y);
        } else { sized = false }
        let sw = self.get_width();
        let sh = self.get_width();
        for w in &self.widgets {
            let (dx, dy) = w.get_location();
            if !sized
            || (dx + w.get_width() < sw
            || dy + w.get_height() < sh) {
                w.draw(canvas, width, x+dx, y+dy);
            }
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
            self.justify(self.alignment);
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
            alignment: Alignment::Center,
        }
    }
    pub fn new_with_spacing(spacing: u32) -> Self {
        Wbox {
            spacing,
            background: None,
            widgets: Vec::new(),
            orientation: Orientation::Horizontal,
            alignment: Alignment::Start,
        }
    }
    pub fn from(background: impl Widget + 'static) -> Self {
        Wbox {
            spacing: 0,
            background: Some(Rc::new(background)),
            widgets: Vec::new(),
            orientation: Orientation::Horizontal,
            alignment: Alignment::Start,
        }
    }
    pub fn new_with_size(w: u32, h: u32) -> Self {
        Wbox {
            spacing: 0,
            background: Some(Rc::new(Rectangle::empty(w, h))),
            widgets: Vec::new(),
            orientation: Orientation::Horizontal,
            alignment: Alignment::Start,
        }
    }
    pub fn new_with_orientation(orientation: Orientation) -> Self {
        Wbox {
            spacing: 0,
            background: None,
            widgets: Vec::new(),
            alignment: Alignment::Start,
            orientation,
        }
    }
    pub fn justify(&mut self, alignment: Alignment) {
        match alignment {
            Alignment::Start => match self.orientation {
                Orientation::Horizontal => {
                    for w in &mut self.widgets {
                        let (x, _) = w.get_location();
                        w.set_location(x, 0);
                    }
                }
                Orientation::Vertical => {
                    for w in &mut self.widgets {
                        let (_, y) = w.get_location();
                        w.set_location(0, y);
                    }
                }
            }
            Alignment::Center => match self.orientation {
                Orientation::Horizontal => {
                    let height = self.get_height();
                    for w in &mut self.widgets {
                        let (x, _) = w.get_location();
                        w.set_location(x, (height - w.get_height())/2);
                    }
                }
                Orientation::Vertical => {
                    let width = self.get_width();
                    for w in &mut self.widgets {
                        let (_, y) = w.get_location();
                        w.set_location((width - w.get_width())/2, y);
                    }
                }
            }
            Alignment::End => match self.orientation {
                Orientation::Horizontal => {
                    let height = self.get_height();
                    for w in &mut self.widgets {
                        let (x, _) = w.get_location();
                        w.set_location(x, height - w.get_height());
                    }
                }
                Orientation::Vertical => {
                    let width = self.get_width();
                    for w in &mut self.widgets {
                        let (_, y) = w.get_location();
                        w.set_location(width - w.get_width(), y);
                    }
                }
            }
        }
    }
    pub fn center(&mut self, widget: impl Widget + 'static) -> Result<(), Error> {
        anchor(self, widget, Anchor::Center, 0)
    }
    pub fn set_spacing(&mut self, spacing: u32) {
        self.spacing = spacing;
    }
    pub fn set_background(&mut self, background: impl Widget + 'static) {
        self.background = Some(Rc::new(background));
    }
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.alignment = alignment;
    }
}

impl Widget for Wbox {
    fn action<'s>(&'s mut self, name: Action, event_loop: &mut Vec<Damage<'s>>, widget_x: u32, widget_y: u32) {
        for l in &mut self.widgets {
            let (dx, dy) = l.get_location();
            l.action(name, event_loop, widget_x + dx, widget_y + dy)
        }
    }
}
