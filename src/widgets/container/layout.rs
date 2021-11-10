use crate::*;
use scene::{Region, RenderNode};

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

pub struct WidgetLayout {
    spacing: f32,
    pub widgets: Vec<Box<dyn Widget>>,
    orientation: Orientation,
    alignment: Alignment,
}

impl Geometry for WidgetLayout {
    fn width(&self) -> f32 {
        let mut width = 0.;
        match self.orientation {
            Orientation::Horizontal => {
                for w in &self.widgets {
                    let lwidth = w.width();
                    width += lwidth + self.spacing;
                }
                if width > self.spacing {
                    width -= self.spacing
                }
            }
            Orientation::Vertical => {
                for w in &self.widgets {
                    let lwidth = w.width();
                    if lwidth > width {
                        width = lwidth;
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
                    let lheight = w.height();
                    if lheight > height {
                        height = lheight;
                    }
                }
            }
            Orientation::Vertical => {
                for w in &self.widgets {
                    let lheight = w.height();
                    height += lheight + self.spacing;
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
            match self.orientation {
                Orientation::Horizontal => {
                    match self.alignment {
                        Alignment::Start => dy = 0.,
                        Alignment::Center => dy = (sh - wh) / 2.,
                        Alignment::End => dy = sh - wh,
                    }
                    w.draw(ctx, x + dx, y + dy);
                    dx += w.width() + self.spacing;
                }
                Orientation::Vertical => {
                    match self.alignment {
                        Alignment::Start => dx = 0.,
                        Alignment::Center => dx = (sw - ww) / 2.,
                        Alignment::End => dx = sw - ww,
                    }
                    w.draw(ctx, x + dx, y + dy);
                    dy += w.height() + self.spacing;
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
        self.widgets.push(Box::new(widget));
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
}

impl Widget for WidgetLayout {
    fn create_node(&self, x: f32, y: f32) -> RenderNode {
        let sw = self.width();
        let sh = self.height();
        let mut nodes = Vec::new();
        let (mut dx, mut dy) = (0., 0.);

        for w in &self.widgets {
            let ww = w.width();
            let wh = w.height();
            match self.orientation {
                Orientation::Horizontal => {
                    match self.alignment {
                        Alignment::Start => dy = 0.,
                        Alignment::Center => dy = (sh - wh) / 2.,
                        Alignment::End => dy = sh - wh,
                    }
                    nodes.push(w.create_node(x + dx, y + dy));
                    dx += w.width() + self.spacing;
                }
                Orientation::Vertical => {
                    match self.alignment {
                        Alignment::Start => dx = 0.,
                        Alignment::Center => dx = (sw - ww) / 2.,
                        Alignment::End => dx = sw - ww,
                    }
                    nodes.push(w.create_node(x + dx, y + dy));
                    dy += w.height() + self.spacing;
                }
            }
        }

        RenderNode::Container(Region::new(x, y, self.width(), self.height()), nodes)
    }
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, ctx: &mut Context, dispatch: &Dispatch) {
        let sw = self.width();
        let sh = self.height();
        let (mut dx, mut dy) = (0., 0.);

        for w in self.widgets.iter_mut() {
            let ww = w.width();
            let wh = w.height();
            match self.orientation {
                Orientation::Horizontal => {
                    match self.alignment {
                        Alignment::Start => dy = 0.,
                        Alignment::Center => dy = (sh - wh) / 2.,
                        Alignment::End => dy = sh - wh,
                    }
                    w.roundtrip(wx + dx, wy + dy, ctx, dispatch);
                    dx += w.width() + self.spacing;
                }
                Orientation::Vertical => {
                    match self.alignment {
                        Alignment::Start => dx = 0.,
                        Alignment::Center => dx = (sw - ww) / 2.,
                        Alignment::End => dx = sw - ww,
                    }
                    w.roundtrip(wx + dx, wy + dy, ctx, dispatch);
                    dy += w.height() + self.spacing;
                }
            }
        }
    }
}
