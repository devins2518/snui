use crate::*;
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
    size: (u32, u32),
    pub widgets: Vec<Inner>,
    background: u32,
    orientation: Orientation,
    anchor: Anchor,
}

impl Geometry for Wbox {
    fn get_width(&self) -> u32 {
        let mut width = 0;
        for w in &self.widgets {
            if w.is_mapped() {
                let lwidth = w.get_width();
                let (lx, _) = w.coords();
                if lx + lwidth > width {
                    width = lx + lwidth;
                }
            }
        }
        if width < self.size.0 {
            self.size.0
        } else {
            width
        }
    }
    fn get_height(&self) -> u32 {
        let mut height = 0;
        for w in &self.widgets {
            if w.is_mapped() {
                let (_, ly) = w.coords();
                let lheight = w.get_height();
                if ly + lheight > height {
                    height = ly + lheight;
                }
            }
        }
        if height < self.size.1 {
            self.size.1
        } else {
            height
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
        self.size = (width, height);
        Ok(())
    }
}

impl Drawable for Wbox {
    fn set_color(&mut self, color: u32) {
        self.background = color;
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        let sw = self.get_width();
        let sh = self.get_height();
        if self.background != 0 {
            let rectangle =  Rectangle::new(sw, sh, self.background);
            rectangle.draw(canvas, width, x, y);
        }
        for w in &self.widgets {
            if let Ok((dx, dy)) = w.get_location(sw, sh) {
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
        let (mut x, mut y) = (0, 0);
        for w in self.widgets.iter().rev() {
            if w.is_mapped() {
                let (lx, ly) = w.get_location(self.get_width(), self.get_height()).unwrap();
                x = lx;
                y = ly;
                match self.orientation {
                    Orientation::Horizontal => {
                        x += w.get_width() + self.spacing;
                    }
                    Orientation::Vertical => {
                        y += w.get_height() + self.spacing;
                    }
                }
                break;
            }
        }
        let mut i = Inner::new_at(widget, self.anchor, x, y);
        i.map();
        self.widgets.push(i);
        Ok(())
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
            size: (0, 0),
            background: 0,
            widgets: Vec::new(),
            anchor: Anchor::TopLeft,
        }
    }
    pub fn new_with_spacing(orientation: Orientation, spacing: u32) -> Self {
        Wbox {
            spacing,
            orientation,
            size: (0, 0),
            background: 0,
            widgets: Vec::new(),
            anchor: Anchor::TopLeft,
        }
    }
    pub fn from(orientation: Orientation, background: u32) -> Self {
        Wbox {
            spacing: 0,
            background,
            orientation,
            size: (0, 0),
            widgets: Vec::new(),
            anchor: Anchor::TopLeft,
        }
    }
    pub fn set_spacing(&mut self, spacing: u32) {
        self.spacing = spacing;
    }
    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
        for w in &mut self.widgets {
            if w.is_mapped() {
                w.set_anchor(anchor);
            }
        }
    }
    pub fn reposition(&mut self) {
        let (mut x, mut y) = (0, 0);
        for w in &mut self.widgets {
            if w.is_mapped() {
                w.set_location(x, y);
                match self.orientation {
                    Orientation::Horizontal => x += w.get_width() + self.spacing,
                    Orientation::Vertical   => y += w.get_height() + self.spacing,
                }
            }
        }
    }
    pub fn unmap(&mut self, i: usize) {
        if i < self.widgets.len() {
            self.widgets[i].unmap();
            self.reposition();
        }
    }
    pub fn remap(&mut self, i: usize) {
        if i < self.widgets.len() {
            self.widgets[i].map();
            self.reposition();
        }
    }
}

impl Widget for Wbox {
    fn send_action<'s>(&'s mut self, action: Action) {
        for l in &mut self.widgets {
            l.send_action(action);
        }
    }
}
