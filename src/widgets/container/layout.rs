use crate::*;
use std::rc::Rc;
use crate::widgets::Rectangle;
use crate::widgets::active::pointer;
use crate::widgets::active::command::Command;

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

#[derive(Clone)]
pub struct Element {
    mapped: bool,
    pub widget: Rc<dyn Widget>,
}

impl Element {
    fn new(widget: impl Widget + 'static) -> Element {
        Element {
            mapped: true,
            widget: Rc::new(widget),
        }
    }
}

#[derive(Clone)]
pub struct WidgetLayout {
    spacing: u32,
    pub widgets: Vec<Element>,
    background: u32,
    orientation: Orientation,
    alignment: Alignment,
}

impl Geometry for WidgetLayout {
    fn get_width(&self) -> u32 {
        let mut width = 0;
        match self.orientation {
            Orientation::Horizontal => {
                for w in &self.widgets {
                    if w.mapped {
                        let lwidth = w.widget.get_width();
                        width += lwidth + self.spacing;
                    }
                }
                width -= if !self.is_empty() { self.spacing } else { 0 };
            }
            Orientation::Vertical => {
                for w in &self.widgets {
                    if w.mapped {
                        let lwidth = w.widget.get_width();
                        if lwidth > width {
                            width = lwidth;
                        }
                    }
                }
            }
        }
        width
    }
    fn get_height(&self) -> u32 {
        let mut height = 0;
        match self.orientation {
            Orientation::Horizontal => {
                for w in &self.widgets {
                    if w.mapped {
                        let lheight = w.widget.get_height();
                        if lheight > height {
                            height = lheight;
                        }
                    }
                }
            }
            Orientation::Vertical => {
                for w in &self.widgets {
                    if w.mapped {
                        let lheight = w.widget.get_height();
                        height += lheight + self.spacing;
                    }
                }
                height -= if !self.is_empty() { self.spacing } else { 0 };
            }
        }
        height
    }
    fn contains<'d>(
        &'d mut self,
        widget_x: u32,
        widget_y: u32,
        x: u32,
        y: u32,
        event: pointer::Event,
    ) -> Damage {
        let (mut dx, mut dy) = (0, 0);
        for w in &mut self.widgets {
            if w.mapped {
                let widget_width = w.widget.get_width();
                let widget_height = w.widget.get_height();
                if x > widget_x + dx && y > widget_y + dy {
                    if let Some(widget) = Rc::get_mut(&mut w.widget) {
                        let ev = widget.contains(
                            widget_x + dx,
                            widget_y + dy,
                            x,
                            y,
                            event,
                        );
                        if ev.is_some() {
                            return ev;
                        }
                    }
                }
                match self.orientation {
                    Orientation::Horizontal => {
                        dx += widget_width + self.spacing;
                    }
                    Orientation::Vertical => {
                        dy += widget_height + self.spacing;
                    }
                }
            }
        }
        Damage::None
    }
    fn resize(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        Ok(())
    }
}

impl Drawable for WidgetLayout {
    fn set_color(&mut self, color: u32) {
        self.background = color;
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        let sw = self.get_width();
        let sh = self.get_height();
        if self.background != 0 {
            let rectangle = Rectangle::new(sw, sh, self.background);
            rectangle.draw(canvas, width, x, y);
        }
        let (mut dx, mut dy) = (0, 0);
        for w in &self.widgets {
            if w.mapped {
                match self.orientation {
                    Orientation::Horizontal => {
                        match self.alignment {
                            Alignment::Start => dy = 0,
                            Alignment::Center => dy = (sh - w.widget.get_height()) / 2,
                            Alignment::End => dy = sh - w.widget.get_height(),
                        }
                        w.widget.draw(canvas, width, x + dx, y + dy);
                        dx += w.widget.get_width() + self.spacing;
                    }
                    Orientation::Vertical => {
                        match self.alignment {
                            Alignment::Start => dx = 0,
                            Alignment::Center => dx = (sw - w.widget.get_width()) / 2,
                            Alignment::End => dx = sw - w.widget.get_width(),
                        }
                        w.widget.draw(canvas, width, x + dx, y + dy);
                        dy += w.widget.get_height() + self.spacing;
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
    fn get_child(&self) -> Result<&dyn Widget, Error> {
        Err(Error::Message("get_child is not valid on \"wbox\""))
    }
}

impl WidgetLayout {
    pub fn new(orientation: Orientation) -> Self {
        WidgetLayout {
            spacing: 0,
            orientation,
            background: 0,
            widgets: Vec::new(),
            alignment: Alignment::Start,
        }
    }
    pub fn horizontal(spacing: u32) -> Self {
        WidgetLayout {
            spacing,
            orientation: Orientation::Horizontal,
            background: 0,
            widgets: Vec::new(),
            alignment: Alignment::Start,
        }
    }
    pub fn vertical(spacing: u32) -> Self {
        WidgetLayout {
            spacing,
            orientation: Orientation::Vertical,
            background: 0,
            widgets: Vec::new(),
            alignment: Alignment::Start,
        }
    }
    pub fn new_with_spacing(orientation: Orientation, spacing: u32) -> Self {
        WidgetLayout {
            spacing,
            orientation,
            background: 0,
            widgets: Vec::new(),
            alignment: Alignment::Start,
        }
    }
    pub fn from(orientation: Orientation, background: u32) -> Self {
        WidgetLayout {
            spacing: 0,
            background,
            orientation,
            widgets: Vec::new(),
            alignment: Alignment::Start,
        }
    }
    pub fn set_spacing(&mut self, spacing: u32) {
        self.spacing = spacing;
    }
    pub fn set_alignment(&mut self, alignment: Alignment) {
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
    fn send_command<'s>(&'s mut self, command: Command) -> Damage {
        let width = self.get_width();
        let height = self.get_height();
        let (mut dx, mut dy) = (0, 0);
        for w in &mut self.widgets {
            if w.mapped {
                match self.orientation {
                    Orientation::Horizontal => {
                        match self.alignment {
                            Alignment::Start => dy = 0,
                            Alignment::Center => dy = (height - w.widget.get_height()) / 2,
                            Alignment::End => dy = height - w.widget.get_height(),
                        }
                        dx += w.widget.get_width() + self.spacing;
                    }
                    Orientation::Vertical => {
                        match self.alignment {
                            Alignment::Start => dx = 0,
                            Alignment::Center => dx = (width - w.widget.get_width()) / 2,
                            Alignment::End => dx = width - w.widget.get_width(),
                        }
                        dy += w.widget.get_height() + self.spacing;
                    }
                }
            }
            let ev = Rc::get_mut(&mut w.widget).unwrap().send_command(command);
            if ev.is_some() {
                return ev.shift(dx, dy);
            }
        }
        Damage::None
    }
}
