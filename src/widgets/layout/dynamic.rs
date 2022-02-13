//! A layout widget that supports interactive resize.

use crate::widgets::{layout::*, Alignment, CENTER, END, START};
use crate::*;
use scene::RenderNode;

pub struct DynamicLayout<W> {
    orientation: Orientation,
    widgets: Vec<Positioner<Proxy<W>>>,
}

impl<W> FromIterator<W> for DynamicLayout<W> {
    fn from_iter<T: IntoIterator<Item = W>>(iter: T) -> Self {
        let mut this = DynamicLayout::new();
        for widget in iter {
            this.widgets.push(child(widget));
        }
        this
    }
}

impl<D, W> Container<D, W> for DynamicLayout<W>
where
    W: Widget<D>,
{
    fn len(&self) -> usize {
        self.widgets.len()
    }
    fn add(&mut self, widget: W) {
        self.widgets.push(child(widget))
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

impl<W: Geometry> Geometry for DynamicLayout<W> {
    fn width(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => self.widgets.iter().map(|widget| widget.width()).sum(),
            Orientation::Vertical => self
                .widgets
                .iter()
                .map(|widget| widget.width())
                .reduce(|previous, current| previous.max(current))
                .unwrap_or_default(),
        }
    }
    fn height(&self) -> f32 {
        match self.orientation {
            Orientation::Vertical => self.widgets.iter().map(|widget| widget.height()).sum(),
            Orientation::Horizontal => self
                .widgets
                .iter()
                .map(|widget| widget.height())
                .reduce(|previous, current| previous.max(current))
                .unwrap_or_default(),
        }
    }
}

impl<D, W: Widget<D>> Widget<D> for DynamicLayout<W> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        RenderNode::Container(
            self.widgets
                .iter_mut()
                .map(|widget| widget.create_node(transform))
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
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        let mut delta = 0.;
        let len = self.len();
        match self.orientation {
            Orientation::Vertical => {
                let mut dy = 0.;
                self.widgets
                    .iter_mut()
                    .enumerate()
                    .map(move |(i, widget)| {
                        widget.set_coords(0., dy);
                        let min_height = constraints.minimum_height() / (len - i) as f32 + delta;
                        let (width, height) = widget
                            .layout(
                                ctx,
                                &constraints.with_min(
                                    constraints.minimum_width(),
                                    min_height.min(constraints.maximum_height()) - dy,
                                ),
                            )
                            .into();
                        delta = min_height - height;
                        dy += height;
                        Size::new(width, height)
                    })
                    .reduce(|accum, size| {
                        Size::new(accum.width.max(size.width), accum.height + size.height)
                    })
                    .unwrap_or_default()
            }
            Orientation::Horizontal => {
                let mut dx = 0.;
                let f = self
                    .widgets
                    .iter_mut()
                    .enumerate()
                    .map(move |(i, widget)| {
                        let min_width = constraints.minimum_width() / (len - i) as f32 + delta;
                        widget.set_coords(dx, 0.);
                        let (width, height) = widget
                            .layout(
                                ctx,
                                &constraints.with_min(
                                    min_width.min(constraints.maximum_width()) - dx,
                                    constraints.minimum_height(),
                                ),
                            )
                            .into();
                        delta = min_width - width;
                        dx += width;
                        Size::new(width, height)
                    })
                    .reduce(|accum, size| {
                        Size::new(accum.width + size.width, accum.height.max(size.height))
                    })
                    .unwrap_or_default();
                f
            }
        }
    }
}

impl<D> Default for DynamicLayout<Box<dyn Widget<D>>> {
    fn default() -> Self {
        Self {
            widgets: Vec::new(),
            orientation: Orientation::Horizontal,
        }
    }
}

impl<D> DynamicLayout<Box<dyn Widget<D>>> {
    pub fn add<W: Widget<D> + 'static>(&mut self, widget: W) {
        self.widgets.push(child(Box::new(widget)));
    }
}

impl<W: Geometry> DynamicLayout<WidgetBox<W>> {
    pub fn set_alignment(&mut self, alignment: Alignment) {
        match self.orientation {
            Orientation::Horizontal => {
                for widget in &mut self.widgets {
                    match alignment {
                        Alignment::Start => widget.set_anchor(START, CENTER),
                        Alignment::Center => widget.set_anchor(CENTER, CENTER),
                        Alignment::End => widget.set_anchor(END, CENTER),
                    }
                }
            }
            Orientation::Vertical => {
                for widget in &mut self.widgets {
                    match alignment {
                        Alignment::Start => widget.set_anchor(CENTER, START),
                        Alignment::Center => widget.set_anchor(CENTER, CENTER),
                        Alignment::End => widget.set_anchor(CENTER, END),
                    }
                }
            }
        }
    }
    pub fn align(mut self, alignment: Alignment) -> Self {
        self.set_alignment(alignment);
        self
    }
}

impl<W> DynamicLayout<W> {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            orientation: Orientation::Horizontal,
        }
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn clear(&mut self) {
        self.widgets.clear();
    }
    pub fn inner(&mut self) -> &mut [Positioner<Proxy<W>>] {
        self.widgets.as_mut_slice()
    }
}
