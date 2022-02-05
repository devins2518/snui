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
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        let real_width: f32;
        match self.orientation {
            Orientation::Horizontal => {
                real_width = apply_width(&mut self.widgets, width);
            }
            Orientation::Vertical => {
                real_width = self
                    .widgets
                    .iter_mut()
                    .map(|widget| widget.set_width(width).err().unwrap_or(width))
                    .reduce(|acc, width| acc.max(width))
                    .unwrap_or_default();
            }
        }
        real_width.eq(&width).then(|| ()).ok_or(real_width)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        let real_height: f32;
        match self.orientation {
            Orientation::Vertical => {
                real_height = apply_height(&mut self.widgets, height);
            }
            Orientation::Horizontal => {
                real_height = self
                    .widgets
                    .iter_mut()
                    .map(|widget| widget.set_height(height).err().unwrap_or(height))
                    .reduce(|acc, width| acc.max(width))
                    .unwrap_or_default();
            }
        }
        real_height.eq(&height).then(|| ()).ok_or(real_height)
    }
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
    fn maximum_height(&self) -> f32 {
        match self.orientation {
            Orientation::Vertical => self
                .widgets
                .iter()
                .map(|widget| widget.maximum_height())
                .sum(),
            Orientation::Horizontal => self
                .widgets
                .iter()
                .map(|widget| widget.maximum_height())
                .reduce(|accum, height| accum.max(height))
                .unwrap_or_default(),
        }
    }
    fn maximum_width(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => self
                .widgets
                .iter()
                .map(|widget| widget.maximum_width())
                .sum(),
            Orientation::Vertical => self
                .widgets
                .iter()
                .map(|widget| widget.width())
                .reduce(|accum, width| accum.max(width))
                .unwrap_or_default(),
        }
    }
    fn minimum_height(&self) -> f32 {
        match self.orientation {
            Orientation::Vertical => self
                .widgets
                .iter()
                .map(|widget| widget.minimum_height())
                .sum(),
            Orientation::Horizontal => self
                .widgets
                .iter()
                .map(|widget| widget.minimum_height())
                .reduce(|accum, height| accum.max(height))
                .unwrap_or_default(),
        }
    }
    fn minimum_width(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => self
                .widgets
                .iter()
                .map(|widget| widget.minimum_width())
                .sum(),
            Orientation::Vertical => self
                .widgets
                .iter()
                .map(|widget| widget.width())
                .reduce(|accum, width| accum.max(width))
                .unwrap_or_default(),
        }
    }
}

impl<D, W: Widget<D>> Widget<D> for DynamicLayout<W> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let sw = self.width();
        let sh = self.height();
        let (mut dx, mut dy) = (0., 0.);
        RenderNode::Container(
            self.widgets
                .iter_mut()
                .map(|child| {
                    let node;
                    child.set_coords(dx, dy);
                    match self.orientation {
                        Orientation::Horizontal => {
                            let _ = child.set_height(sh);
                            node = child.create_node(transform);
                            dx += child.width();
                        }
                        Orientation::Vertical => {
                            let _ = child.set_width(sw);
                            node = child.create_node(transform);
                            dy += child.height();
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
