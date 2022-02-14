//! A layout widget that supports interactive resize.

use crate::widgets::{layout::*, Alignment, CENTER, END, START};
use crate::*;
use scene::{Region, RenderNode};

pub struct DynamicLayout<W> {
    size: Size,
    orientation: Orientation,
    children: Vec<Positioner<Proxy<W>>>,
}

impl<W> FromIterator<W> for DynamicLayout<W> {
    fn from_iter<T: IntoIterator<Item = W>>(iter: T) -> Self {
        let mut this = DynamicLayout::new();
        for widget in iter {
            this.children.push(child(widget));
        }
        this
    }
}

impl<D, W> Container<D, W> for DynamicLayout<W>
where
    W: Widget<D>,
{
    fn len(&self) -> usize {
        self.children.len()
    }
    fn add(&mut self, widget: W) {
        self.children.push(child(widget))
    }
    fn remove(&mut self, index: usize) -> W {
        self.children.remove(index).widget.inner
    }
    fn children(&mut self) -> Vec<&mut W> {
        self.children
            .iter_mut()
            .map(|inner| inner.widget.deref_mut())
            .collect()
    }
}

impl<W: Geometry> Geometry for DynamicLayout<W> {
    fn width(&self) -> f32 {
        self.size.width
    }
    fn height(&self) -> f32 {
        self.size.height
    }
}

impl<D, W: Widget<D>> Widget<D> for DynamicLayout<W> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        RenderNode::Container {
            bound: Region::from_transform(transform, self.size.width, self.size.height),
            children: self
                .children
                .iter_mut()
                .map(|widget| widget.create_node(transform))
                .collect(),
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        self.children
            .iter_mut()
            .map(|widget| widget.sync(ctx, event))
            .max()
            .unwrap_or_default()
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        let mut delta = 0.;
        let len = self.len();
        self.size = match self.orientation {
            Orientation::Vertical => {
                let mut dy = 0.;
                self.children
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
                self.children
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
                    .unwrap_or_default()
            }
        };
        self.size
    }
}

impl<D> Default for DynamicLayout<Box<dyn Widget<D>>> {
    fn default() -> Self {
        Self {
            size: Size::default(),
            children: Vec::new(),
            orientation: Orientation::Horizontal,
        }
    }
}

impl<D> DynamicLayout<Box<dyn Widget<D>>> {
    pub fn add<W: Widget<D> + 'static>(&mut self, widget: W) {
        self.children.push(child(Box::new(widget)));
    }
}

impl<W: Geometry> DynamicLayout<WidgetBox<W>> {
    pub fn set_alignment(&mut self, alignment: Alignment) {
        match self.orientation {
            Orientation::Horizontal => {
                for widget in &mut self.children {
                    match alignment {
                        Alignment::Start => widget.set_anchor(START, CENTER),
                        Alignment::Center => widget.set_anchor(CENTER, CENTER),
                        Alignment::End => widget.set_anchor(END, CENTER),
                    }
                }
            }
            Orientation::Vertical => {
                for widget in &mut self.children {
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
            size: Size::default(),
            children: Vec::new(),
            orientation: Orientation::Horizontal,
        }
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn clear(&mut self) {
        self.children.clear();
    }
    pub fn inner(&mut self) -> &mut [Positioner<Proxy<W>>] {
        self.children.as_mut_slice()
    }
}
