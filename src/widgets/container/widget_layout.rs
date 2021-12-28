use crate::widgets::container::{Child, Container};
use crate::widgets::Alignment;
use crate::*;
use scene::{Coords, Region, RenderNode};

pub struct WidgetLayout<R> {
    spacing: f32,
    widgets: Vec<Child<R>>,
    alignment: Alignment,
    orientation: Orientation,
}

impl<R> FromIterator<Child<R>> for WidgetLayout<R> {
    fn from_iter<T: IntoIterator<Item = Child<R>>>(iter: T) -> Self {
        let mut layout = WidgetLayout::new(0.);
        for c in iter {
            layout.widgets.push(c);
        }
        layout
    }
}

impl<R> Geometry for WidgetLayout<R> {
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

impl<R: 'static> Container<R> for WidgetLayout<R> {
    fn len(&self) -> usize {
        self.widgets.len()
    }
    fn add(&mut self, widget: impl Widget<R> + 'static) {
        self.widgets.push(Child::new(widget));
    }
    fn remove(&mut self, index: usize) -> Child<R> {
        self.widgets.remove(index)
    }
}

impl<R> WidgetLayout<R> {
    pub fn new<S: Into<f32>>(spacing: S) -> Self {
        WidgetLayout {
            spacing: spacing.into(),
            widgets: Vec::new(),
            alignment: Alignment::Start,
            orientation: Orientation::Horizontal,
        }
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn spacing<S: Into<f32>>(mut self, spacing: S) -> Self {
        self.spacing = spacing.into();
        self
    }
    pub fn set_spacing<S: Into<f32>>(&mut self, spacing: S) {
        self.spacing = spacing.into();
    }
    pub fn justify(&mut self, alignment: Alignment) {
        self.alignment = alignment;
    }
    pub fn clear(&mut self) {
        self.widgets = Vec::new();
    }
}

impl<R> Widget<R> for WidgetLayout<R> {
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
                            child.coords = Coords::new(dx, dy);
                            node = child.create_node_ext(x, y, ww, sh);
                            dx += child.width() + spacing;
                        }
                        Orientation::Vertical => {
                            match alignment {
                                Alignment::Start => dx = 0.,
                                Alignment::Center => dx = ((sw - ww) / 2.).floor(),
                                Alignment::End => dx = sw - ww,
                            }
                            child.coords = Coords::new(dx, dy);
                            node = child.create_node_ext(x, y, sw, wh);
                            dy += child.height() + spacing;
                        }
                    }
                    node
                })
                .collect(),
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<R>, event: &Event<R>) -> Damage {
        let mut damage = Damage::None;
        for child in self.widgets.iter_mut() {
            damage = damage.max(child.sync(ctx, event));
        }
        damage
    }
}
