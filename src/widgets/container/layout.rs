use crate::*;
use scene::{Coords, RenderNode};

#[derive(Copy, Clone, Debug)]
pub enum Alignment {
    Start,
    Center,
    End,
}

struct Child {
    coords: Coords,
    widget: Box<dyn Widget>,
}

impl Child {
    fn new(widget: impl Widget + 'static) -> Self {
        Child {
            coords: Coords::new(0., 0.),
            widget: Box::new(widget),
        }
    }
}

impl Geometry for Child {
    fn width(&self) -> f32 {
        self.widget.width()
    }
    fn height(&self) -> f32 {
        self.widget.height()
    }
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        self.widget.set_size(width, height)
    }
}

pub struct WidgetLayout {
    spacing: f32,
    widgets: Vec<Child>,
    alignment: Alignment,
    orientation: Orientation,
}

impl Geometry for WidgetLayout {
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        // To-do
        Err((self.width(), self.height()))
    }
    fn width(&self) -> f32 {
        let mut width = 0.;
        match self.orientation {
            Orientation::Horizontal => {
                for w in &self.widgets {
                    width += w.width() + self.spacing;
                }
                width -= self.spacing.min(width);
            }
            Orientation::Vertical => {
                for w in &self.widgets {
                    width = width.max(w.width());
                }
            }
        }
        width
    }
    fn height(&self) -> f32 {
        let mut height = 0.;
        match self.orientation {
            Orientation::Vertical => {
                for w in &self.widgets {
                    height += w.height() + self.spacing;
                }
                height -= self.spacing.min(height);
            }
            Orientation::Horizontal => {
                for w in &self.widgets {
                    height = height.max(w.height());
                }
            }
        }
        height
    }
}

impl Container for WidgetLayout {
    fn len(&self) -> usize {
        self.widgets.len()
    }
    fn add(&mut self, widget: impl Widget + 'static) {
        self.widgets.push(Child::new(widget));
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
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let sw = self.width();
        let sh = self.height();
        let spacing = self.spacing;
        let orientation = self.orientation;
        let alignment = self.alignment;
        let (mut dx, mut dy) = (0., 0.);
        RenderNode::Container(
            self.widgets
                .iter_mut()
                .map(|child| {
                    let node;
                    let ww = child.width();
                    let wh = child.height();
                    match orientation {
                        Orientation::Horizontal => {
                            match alignment {
                                Alignment::Start => dy = 0.,
                                Alignment::Center => dy = (sh - wh) / 2.,
                                Alignment::End => dy = sh - wh,
                            }
                            node = child.widget.create_node(x + dx, y + dy);
                            child.coords = Coords::new(dx, dy);
                            dx += child.width() + spacing;
                        }
                        Orientation::Vertical => {
                            match alignment {
                                Alignment::Start => dx = 0.,
                                Alignment::Center => dx = (sw - ww) / 2.,
                                Alignment::End => dx = sw - ww,
                            }
                            node = child.widget.create_node(x + dx, y + dy);
                            child.coords = Coords::new(dx, dy);
                            dy += child.height() + spacing;
                        }
                    }
                    node
                })
                .collect(),
        )
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        // To-do
        for child in self.widgets.iter_mut() {
            if let Event::Pointer(mut x, mut y, p) = event {
                x -= child.coords.x;
                y -= child.coords.y;
                child.widget.sync(ctx, Event::Pointer(x, y, p));
            } else {
                child.widget.sync(ctx, event)
            }
        }
    }
}
