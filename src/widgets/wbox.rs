use crate::*;
use std::rc::Rc;
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
    anchor: Anchor,
}

impl Geometry for Wbox {
    fn get_width(&self) -> u32 {
        if let Some(bg) = &self.background {
            bg.get_width()
        } else {
            let mut x = 0;
            let mut width = 0;
            for w in &self.widgets {
                if w.is_mapped() {
                    let lwidth = w.get_width();
                    let (lx, _) = w.coords();
                    if lx + lwidth > x + width {
                        x = lx;
                        width = lwidth;
                    }
                }
            }
            return x + width
        }
    }
    fn get_height(&self) -> u32 {
        if let Some(bg) = &self.background {
            bg.get_height()
        } else {
            let mut y = 0;
            let mut height = 0;
            for w in &self.widgets {
                if w.is_mapped() {
                    let (_, ly) = w.coords();
                    let lheight = w.get_height();
                    if ly + lheight > y + height {
                        y = ly;
                        height = lheight;
                    }
                }
            }
            return y + height
        }
    }
    fn contains<'d>(&'d mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        let width = self.get_width();
        let height = self.get_height();
        for l in &mut self.widgets {
            let (dx, dy) = l.get_location(width, height).unwrap();
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
            if let Ok((dx, dy)) = w.get_location(self.get_width(), self.get_height()) {
                if !sized
                || (dx + w.get_width() < sw
                || dy + w.get_height() < sh) {
                    w.draw(canvas, width, x+dx, y+dy);
                }
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
            let (mut x, mut y) = w.get_location(self.get_width(), self.get_height()).unwrap();
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
                let mut i = Inner::new_at(widget, self.anchor, x, y);
                i.map();
                self.widgets.push(i);
                Ok(())
            } else {
                Err(Error::Dimension("wbox", widget.get_width(), widget.get_height()))
            }
        } else {
            let mut i = Inner::new_at(widget, self.anchor, x, y);
            i.map();
            self.widgets.push(i);
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
    pub fn new(orientation: Orientation) -> Self {
        Wbox {
            spacing: 0,
            orientation,
            background: None,
            widgets: Vec::new(),
            anchor: Anchor::TopLeft,
        }
    }
    pub fn new_with_spacing(orientation: Orientation, spacing: u32) -> Self {
        Wbox {
            spacing,
            orientation,
            background: None,
            widgets: Vec::new(),
            anchor: Anchor::TopLeft,
        }
    }
    pub fn from(orientation: Orientation, background: impl Widget + 'static) -> Self {
        Wbox {
            spacing: 0,
            orientation,
            widgets: Vec::new(),
            anchor: Anchor::TopLeft,
            background: Some(Rc::new(background)),
        }
    }
    pub fn new_with_size(orientation: Orientation, w: u32, h: u32) -> Self {
        Wbox {
            spacing: 0,
            orientation,
            widgets: Vec::new(),
            anchor: Anchor::TopLeft,
            background: Some(Rc::new(Rectangle::empty(w, h))),
        }
    }
    pub fn set_spacing(&mut self, spacing: u32) {
        self.spacing = spacing;
    }
    pub fn set_background(&mut self, background: impl Widget + 'static) {
        self.background = Some(Rc::new(background));
    }
    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
        for w in &mut self.widgets {
            if w.is_mapped() {
                w.set_anchor(anchor);
            }
        }
    }
}

impl Widget for Wbox {
    fn send_action<'s>(&'s mut self, action: Action, event_loop: &mut Vec<Damage>, widget_x: u32, widget_y: u32) {
        let width = self.get_width();
        let height = self.get_height();
        for l in &mut self.widgets {
            let (dx, dy) = l.get_location(width, height).unwrap();
            l.send_action(action, event_loop, widget_x + dx, widget_y + dy)
        }
    }
}
