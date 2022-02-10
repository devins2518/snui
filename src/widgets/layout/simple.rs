//! A layout widget that handles spacing and alignment.

use crate::widgets::layout::{child, Container, Positioner};
use crate::widgets::Alignment;
use crate::*;
use scene::RenderNode;

#[derive(Debug)]
pub struct SimpleLayout<W> {
    spacing: f32,
    widgets: Vec<Positioner<Proxy<W>>>,
    alignment: Alignment,
    orientation: Orientation,
}

impl<W> FromIterator<W> for SimpleLayout<W> {
    fn from_iter<T: IntoIterator<Item = W>>(iter: T) -> Self {
        let mut layout = SimpleLayout::new();
        for widget in iter {
            layout.widgets.push(child(widget));
        }
        layout
    }
}

impl<D, W> Container<D, W> for SimpleLayout<W>
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

impl<W: Geometry> Geometry for SimpleLayout<W> {
    fn width(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => {
                self.widgets
                    .iter()
                    .map(|widget| widget.width())
                    .sum::<f32>()
                    + (self.widgets.len() as f32 - 1.) * self.spacing
            }
            Orientation::Vertical => self
                .widgets
                .iter()
                .map(|widget| widget.width())
                .reduce(|accum, width| accum.max(width))
                .unwrap_or_default(),
        }
    }
    fn height(&self) -> f32 {
        match self.orientation {
            Orientation::Vertical => {
                self.widgets
                    .iter()
                    .map(|widget| widget.height())
                    .sum::<f32>()
                    + (self.widgets.len() as f32 - 1.) * self.spacing
            }
            Orientation::Horizontal => self
                .widgets
                .iter()
                .map(|widget| widget.height())
                .reduce(|accum, height| accum.max(height))
                .unwrap_or_default(),
        }
    }
}

impl<D> Default for SimpleLayout<Box<dyn Widget<D>>> {
    fn default() -> Self {
        SimpleLayout {
            spacing: 0.,
            widgets: Vec::new(),
            alignment: Alignment::Start,
            orientation: Orientation::Horizontal,
        }
    }
}

impl<D> SimpleLayout<Box<dyn Widget<D>>> {
    /// The default behaviour.
    pub fn add<W: Widget<D> + 'static>(&mut self, widget: W) {
        self.widgets.push(child(Box::new(widget)));
    }
}

impl<W> SimpleLayout<W> {
    pub fn new() -> Self {
        SimpleLayout {
            spacing: 0.,
            widgets: Vec::new(),
            alignment: Alignment::Start,
            orientation: Orientation::Horizontal,
        }
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
    pub fn set_spacing(&mut self, spacing: f32) {
        self.spacing = spacing;
    }
    pub fn justify(&mut self, alignment: Alignment) {
        self.alignment = alignment;
    }
    pub fn clear(&mut self) {
        self.widgets.clear();
    }
    pub fn inner(&mut self) -> &mut [Positioner<Proxy<W>>] {
        self.widgets.as_mut_slice()
    }
}

impl<D, W: Widget<D>> Widget<D> for SimpleLayout<W> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
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
        self.widgets
            .iter_mut()
            .map(|widget| widget.sync(ctx, event))
            .max()
            .unwrap_or_default()
    }
    fn prepare_draw(&mut self) {
        for widget in self.widgets.iter_mut() {
            widget.prepare_draw()
        }
    }
}
