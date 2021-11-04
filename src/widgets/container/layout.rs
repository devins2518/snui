use crate::*;

#[derive(Copy, Clone, Debug)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

#[derive(Copy, Clone, Debug)]
pub enum Alignment {
    Start,
    Center,
    End,
}

pub struct Element {
    mapped: bool,
    hidden: bool,
    widget: Box<dyn Widget>,
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

impl Geometry for Element {
    fn width(&self) -> f32 {
        self.widget.width()
    }
    fn height(&self) -> f32 {
        self.widget.height()
    }
}

impl Drawable for Element {
    fn set_color(&mut self, color: u32) {
        self.widget.set_color(color);
    }
    fn draw(&self, ctx: &mut Context, x: f32, y: f32) {
        if !self.hidden {
            self.widget.draw(ctx, x, y);
        }
    }
}

impl Widget for Element {
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, ctx: &mut Context, dispatch: &Dispatch) {
        self.widget.roundtrip(wx, wy, ctx, dispatch);
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
                        let lwidth = w.width();
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
                        let lwidth = w.width();
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
                        let lheight = w.height();
                        if lheight > height {
                            height = lheight;
                        }
                    }
                }
            }
            Orientation::Vertical => {
                for w in &self.widgets {
                    if w.mapped {
                        let lheight = w.height();
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
            let ww = w.width();
            let wh = w.height();
            if w.mapped {
                match self.orientation {
                    Orientation::Horizontal => {
                        match self.alignment {
                            Alignment::Start => dy = 0.,
                            Alignment::Center => dy = (sh - wh) / 2.,
                            Alignment::End => dy = sh - wh,
                        }
                        w.draw(ctx, x + dx, y + dy);
                        dx += w.widget.width() + self.spacing;
                    }
                    Orientation::Vertical => {
                        match self.alignment {
                            Alignment::Start => dx = 0.,
                            Alignment::Center => dx = (sw - ww) / 2.,
                            Alignment::End => dx = sw - ww,
                        }
                        w.draw(ctx, x + dx, y + dy);
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
        let sw = self.width();
        let sh = self.height();
        let mut damage = false;
        let (mut dx, mut dy) = (0., 0.);
        for w in self.widgets.iter_mut() {
            let ww = w.width();
            let wh = w.height();
            if w.mapped {
                match self.orientation {
                    Orientation::Horizontal => {
                        match self.alignment {
                            Alignment::Start => dy = 0.,
                            Alignment::Center => dy = (sh - wh) / 2.,
                            Alignment::End => dy = sh - wh,
                        }
                        let region = Region::new(wx + dx, wy + dy, ww, wh);
                        w.roundtrip(wx + dx, wy + dy, ctx, dispatch);
                        if let DamageType::Partial = ctx.damage_type() {
                            if ww != w.width() || wh != w.height() {
                                damage = true;
                            } else {
                                w.damage(&region, wx + dx, wy + dy, ctx);
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
                        let region = Region::new(wx, wy, ww, wh);
                        w.roundtrip(wx + dx, wy + dy, ctx, dispatch);
                        if let DamageType::Partial = ctx.damage_type() {
                            if ww != w.width() || wh != w.height() {
                                damage = true;
                            } else {
                                w.damage(&region, wx + dx, wy + dy, ctx);
                            }
                        }
                        dy += w.widget.height() + self.spacing;
                    }
                }
            }
        }
        if !damage {
            if self.width() != sw || self.height() != sh {
                self.damage(&Region::new(wx, wy, sw, sh), wx, wy, ctx);
            }
        }
    }
}
