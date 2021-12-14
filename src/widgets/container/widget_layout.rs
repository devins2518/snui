use crate::widgets::Alignment;
use crate::*;
use crate::widgets::container::Child;
use scene::{Coords, Region, RenderNode};

pub struct WidgetLayout {
    spacing: f32,
    widgets: Vec<Child>,
    alignment: Alignment,
    orientation: Orientation,
}

impl Geometry for WidgetLayout {
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
    pub fn from(mut widgets: Vec<impl Widget + 'static>) -> WidgetLayout {
        WidgetLayout {
            spacing: 0.,
            widgets: widgets.drain(0..).map(|w| Child::new(w)).collect(),
            alignment: Alignment::Start,
            orientation: Orientation::Horizontal,
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
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing as f32;
        self
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
        RenderNode::Container {
            region: Region::new(x, y, sw, sh),
            nodes: self
                .widgets
                .iter_mut()
                .map(|child| {
                    let node;
                    let ww = child.width();
                    let wh = child.height();
                    match orientation {
                        Orientation::Horizontal => {
                            match alignment {
                                Alignment::Start => dy = 0.,
                                Alignment::Center => dy = ((sh - wh) / 2.).floor(),
                                Alignment::End => dy = sh - wh,
                            }
                            node = RenderNode::Extension {
                                background: scene::Instruction::empty(x + dx, y + dy, ww, sh),
                                border: None,
                                node: Box::new(child.widget.create_node(x + dx, y + dy)),
                            };
                            child.coords = Coords::new(dx, dy);
                            dx += child.width() + spacing;
                        }
                        Orientation::Vertical => {
                            match alignment {
                                Alignment::Start => dx = 0.,
                                Alignment::Center => dx = ((sw - ww) / 2.).floor(),
                                Alignment::End => dx = sw - ww,
                            }
                            node = RenderNode::Extension {
                                background: scene::Instruction::empty(x + dx, y + dy, sw, wh),
                                border: None,
                                node: Box::new(child.widget.create_node(x + dx, y + dy)),
                            };
                            child.coords = Coords::new(dx, dy);
                            dy += child.height() + spacing;
                        }
                    }
                    node
                })
                .collect(),
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
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