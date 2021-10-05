use crate::*;

#[derive(Copy, Clone, Debug)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

#[derive(Copy, Clone)]
pub enum Alignment {
    Start,
    Center,
    End,
}

pub struct Element {
    mapped: bool,
    pub widget: Box<dyn Widget>,
}

impl Element {
    fn new(widget: impl Widget + 'static) -> Element {
        Element {
            mapped: true,
            widget: Box::new(widget),
        }
    }
}

pub struct WidgetLayout {
    spacing: u32,
    pub widgets: Vec<Element>,
    orientation: Orientation,
    alignment: Alignment,
}

impl Geometry for WidgetLayout {
    fn width(&self) -> u32 {
        let mut width = 0;
        match self.orientation {
            Orientation::Horizontal => {
                for w in &self.widgets {
                    if w.mapped {
                        let lwidth = w.widget.width();
                        width += lwidth + self.spacing;
                    }
                }
                width -= if !self.is_empty() { self.spacing } else { 0 };
            }
            Orientation::Vertical => {
                for w in &self.widgets {
                    if w.mapped {
                        let lwidth = w.widget.width();
                        if lwidth > width {
                            width = lwidth;
                        }
                    }
                }
            }
        }
        width
    }
    fn height(&self) -> u32 {
        let mut height = 0;
        match self.orientation {
            Orientation::Horizontal => {
                for w in &self.widgets {
                    if w.mapped {
                        let lheight = w.widget.height();
                        if lheight > height {
                            height = lheight;
                        }
                    }
                }
            }
            Orientation::Vertical => {
                for w in &self.widgets {
                    if w.mapped {
                        let lheight = w.widget.height();
                        height += lheight + self.spacing;
                    }
                }
                height -= if !self.is_empty() { self.spacing } else { 0 };
            }
        }
        height
    }
}

impl Drawable for WidgetLayout {
    fn set_color(&mut self, _color: u32) { }
    fn draw(&self, canvas: &mut Canvas, x: u32, y: u32) {
        let sw = self.width();
        let sh = self.height();
        let (mut dx, mut dy) = (0, 0);
        for w in &self.widgets {
            if w.mapped {
                match self.orientation {
                    Orientation::Horizontal => {
                        match self.alignment {
                            Alignment::Start => dy = 0,
                            Alignment::Center => dy = (sh - w.widget.height()) / 2,
                            Alignment::End => dy = sh - w.widget.height(),
                        }
                        w.widget.draw(canvas, x + dx, y + dy);
                        dx += w.widget.width() + self.spacing;
                    }
                    Orientation::Vertical => {
                        match self.alignment {
                            Alignment::Start => dx = 0,
                            Alignment::Center => dx = (sw - w.widget.width()) / 2,
                            Alignment::End => dx = sw - w.widget.width(),
                        }
                        w.widget.draw(canvas, x + dx, y + dy);
                        dy += w.widget.height() + self.spacing;
                    }
                }
            }
        }
    }
}

impl Container for WidgetLayout {
    fn len(&self) -> usize {
        self.widgets.len()
    }
    fn add(&mut self, widget: impl Widget + 'static) -> Result<(), Error> {
        self.widgets.push(Element::new(widget));
        Ok(())
    }
}

impl WidgetLayout {
    pub fn new(orientation: Orientation) -> Self {
        WidgetLayout {
            spacing: 0,
            orientation,
            widgets: Vec::new(),
            alignment: Alignment::Start,
        }
    }
    pub fn horizontal(spacing: u32) -> Self {
        WidgetLayout {
            spacing,
            widgets: Vec::new(),
            alignment: Alignment::Start,
            orientation: Orientation::Horizontal,
        }
    }
    pub fn vertical(spacing: u32) -> Self {
        WidgetLayout {
            spacing,
            widgets: Vec::new(),
            alignment: Alignment::Start,
            orientation: Orientation::Vertical,
        }
    }
    pub fn new_with_spacing(orientation: Orientation, spacing: u32) -> Self {
        WidgetLayout {
            spacing,
            orientation,
            widgets: Vec::new(),
            alignment: Alignment::Start,
        }
    }
    pub fn set_spacing(&mut self, spacing: u32) {
        self.spacing = spacing;
    }
    pub fn justify(&mut self, alignment: Alignment) {
        self.alignment = alignment;
    }
    pub fn clear(&mut self) {
        self.widgets = Vec::new();
    }
    pub fn unmap(&mut self, i: usize) {
        if i < self.widgets.len() {
            self.widgets[i].mapped = false;
        }
    }
    pub fn remap(&mut self, i: usize) {
        if i < self.widgets.len() {
            self.widgets[i].mapped = true;
        }
    }
    pub fn remap_all(&mut self) {
        for w in &mut self.widgets {
            w.mapped = true;
        }
    }
}

impl Widget for WidgetLayout {
    fn damaged(&self) -> bool {
        for w in &self.widgets {
            if w.mapped { return true }
        }
        false
    }
    fn roundtrip<'d>(
        &'d mut self,
        widx: u32,
        widy: u32,
        dispatched: &Dispatch,
    ) -> Option<Damage> {
        match dispatched {
            Dispatch::Commit => for w in self.widgets.iter_mut() {
                w.mapped = w.mapped == false;
            }
            _ => {
                let (mut dx, mut dy) = (0, 0);
                for w in &mut self.widgets {
                    if w.mapped {
                        let widwidth = w.widget.width();
                        let widheight = w.widget.height();
                        let ev = w.widget.roundtrip(widx + dx, widy + dy, dispatched);
                        if ev.is_some() {
                            return ev;
                        }
                        match self.orientation {
                            Orientation::Horizontal => {
                                dx += widwidth + self.spacing;
                            }
                            Orientation::Vertical => {
                                dy += widheight + self.spacing;
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
