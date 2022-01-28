use crate::widgets::container::{child, Container, Positioner};
use crate::widgets::shapes::Rectangle;
use crate::widgets::Alignment;
use crate::*;
use scene::Instruction;
use scene::RenderNode;

pub struct WidgetLayout<W> {
    spacing: f32,
    widgets: Vec<Positioner<Proxy<W>>>,
    alignment: Alignment,
    orientation: Orientation,
}

impl<W> FromIterator<W> for WidgetLayout<W> {
    fn from_iter<T: IntoIterator<Item = W>>(iter: T) -> Self {
        let mut layout = WidgetLayout::new(0.);
        for widget in iter {
            layout.widgets.push(child(widget));
        }
        layout
    }
}

impl<D, W> Container<D, W> for WidgetLayout<W>
where
    W: Widget<D>,
{
    fn len(&self) -> usize {
        self.widgets.len()
    }
    fn add(&mut self, widget: W) {
        self.widgets.push(child(widget));
    }
    fn remove(&mut self, index: usize) -> W {
        self.widgets.remove(index).widget.inner
    }
    fn widgets(&mut self) -> Vec<&mut W> {
        self.widgets
            .iter_mut()
            .map(|inner| inner.widget.deref_mut())
            .collect()
    }
}

impl<W: Geometry> Geometry for WidgetLayout<W> {
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

impl<D> Default for WidgetLayout<Box<dyn Widget<D>>> {
    fn default() -> Self {
        WidgetLayout {
            spacing: 0.,
            widgets: Vec::new(),
            alignment: Alignment::Start,
            orientation: Orientation::Horizontal,
        }
    }
}

impl<D> WidgetLayout<Box<dyn Widget<D>>> {
    pub fn add<W: Widget<D> + 'static>(&mut self, widget: W) {
        self.widgets.push(child(Box::new(widget)));
    }
}

impl<W> WidgetLayout<W> {
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

impl<D, W: Widget<D>> Widget<D> for WidgetLayout<W> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let sw = self.width();
        let sh = self.height();
        let spacing = self.spacing;
        let orientation = self.orientation;
        let alignment = self.alignment;
        let (mut dx, mut dy) = (0., 0.);
        RenderNode::Container(
            Instruction::new(transform, Rectangle::empty(sw, sh)),
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
                            child.set_coords(dx, dy);
                            node = child.create_node(transform);
                            dx += child.width() + spacing;
                        }
                        Orientation::Vertical => {
                            match alignment {
                                Alignment::Start => dx = 0.,
                                Alignment::Center => dx = (sw - ww) / 2.,
                                Alignment::End => dx = sw - ww,
                            }
                            child.set_coords(dx, dy);
                            node = child.create_node(transform);
                            dy += child.height() + spacing;
                        }
                    }
                    node
                })
                .collect(),
        )
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        let mut damage = Damage::None;
        for child in self.widgets.iter_mut() {
            damage = damage.max(child.sync(ctx, event));
        }
        damage
    }
}
