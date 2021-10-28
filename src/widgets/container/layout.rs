use crate::context::DamageType;
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
    hidden: bool,
    pub widget: Box<dyn Widget>,
}

impl Element {
    fn new(widget: impl Widget + 'static) -> Element {
        Element {
            mapped: true,
            hidden: false,
            widget: Box::new(widget),
        }
    }
}

pub struct WidgetLayout {
    spacing: f32,
    pub widgets: Vec<Element>,
    orientation: Orientation,
    alignment: Alignment,
}

impl Geometry for WidgetLayout {
    fn width(&self) -> f32 {
        let mut width = 0.;
        match self.orientation {
            Orientation::Horizontal => {
                for w in &self.widgets {
                    if w.mapped {
                        let lwidth = w.widget.width();
                        width += lwidth + self.spacing;
                    }
                }
                if width > self.spacing {
                    width -= self.spacing
                }
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
    fn height(&self) -> f32 {
        let mut height = 0.;
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
                if height > self.spacing {
                    height -= self.spacing
                }
            }
        }
        height
    }
}

impl Drawable for WidgetLayout {
    fn set_color(&mut self, _color: u32) {}
    fn draw(&self, ctx: &mut Context, x: f32, y: f32) {
        let sw = self.width();
        let sh = self.height();
        let (mut dx, mut dy) = (0., 0.);
        for w in &self.widgets {
            if w.mapped {
                match self.orientation {
                    Orientation::Horizontal => {
                        match self.alignment {
                            Alignment::Start => dy = 0.,
                            Alignment::Center => dy = (sh - w.widget.height()) / 2.,
                            Alignment::End => dy = sh - w.widget.height(),
                        }
                        if !w.hidden {
                            w.widget.draw(ctx, x + dx, y + dy);
                        }
                        dx += w.widget.width() + self.spacing;
                    }
                    Orientation::Vertical => {
                        match self.alignment {
                            Alignment::Start => dx = 0.,
                            Alignment::Center => dx = (sw - w.widget.width()) / 2.,
                            Alignment::End => dx = sw - w.widget.width(),
                        }
                        if !w.hidden {
                            w.widget.draw(ctx, x + dx, y + dy);
                        }
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
            spacing: 0.,
            orientation,
            widgets: Vec::new(),
            alignment: Alignment::Start,
        }
    }
    pub fn horizontal(spacing: u32) -> Self {
        WidgetLayout {
            spacing: spacing as f32,
            widgets: Vec::new(),
            alignment: Alignment::Start,
            orientation: Orientation::Horizontal,
        }
    }
    pub fn vertical(spacing: u32) -> Self {
        WidgetLayout {
            spacing: spacing as f32,
            widgets: Vec::new(),
            alignment: Alignment::Start,
            orientation: Orientation::Vertical,
        }
    }
    pub fn new_with_spacing(orientation: Orientation, spacing: u32) -> Self {
        WidgetLayout {
            spacing: spacing as f32,
            orientation,
            widgets: Vec::new(),
            alignment: Alignment::Start,
        }
    }
    pub fn set_spacing(&mut self, spacing: u32) {
        self.spacing = spacing as f32;
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
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, ctx: &mut Context, dispatch: &Dispatch) {
        let mut index = 0;
        let mut draw = false;
        let sw = self.width();
        let sh = self.height();
        let (mut dx, mut dy) = (0., 0.);
        for (i, w) in &mut self.widgets.iter_mut().enumerate() {
            let ww = w.widget.width();
            let wh = w.widget.height();
            if w.mapped {
                match self.orientation {
                    Orientation::Horizontal => {
                        match self.alignment {
                            Alignment::Start => dy = 0.,
                            Alignment::Center => dy = (sh - wh) / 2.,
                            Alignment::End => dy = sh - wh,
                        }
                        w.widget.roundtrip(wx + dx, wy + dy, ctx, dispatch);
                        if let DamageType::Resize = ctx.damage_type() {
                            if ww != w.widget.width() || wh != w.widget.height() {
                                ctx.request_resize();
                            } else {
                                index = i;
                                draw = true;
                            }
                        }
                        dx += w.widget.width() + self.spacing;
                    }
                    Orientation::Vertical => {
                        match self.alignment {
                            Alignment::Start => dx = 0.,
                            Alignment::Center => dx = (sw - ww) / 2.,
                            Alignment::End => dx = sw - ww,
                        }
                        w.widget.roundtrip(wx + dx, wy + dy, ctx, dispatch);
                        if let DamageType::Resize = ctx.damage_type() {
                            if ww != w.widget.width() || wh != w.widget.height() {
                                ctx.request_resize();
                            } else {
                                index = i;
                                draw = true;
                            }
                        }
                        dy += w.widget.height() + self.spacing;
                    }
                }
            }
        }
        if draw {
            for i in 0..index {
                self.widgets[i].hidden = true;
            }
            self.draw(ctx, wx, wy);
            for i in 0..index {
                self.widgets[i].hidden = false;
            }
        }
    }
}
