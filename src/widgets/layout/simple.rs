//! A layout widget that handles spacing and alignment.

use crate::widgets::layout::{child, Container, Positioner};
use crate::widgets::Alignment;
use crate::*;
use scene::{Region, RenderNode};

#[derive(Debug)]
pub struct SimpleLayout<W> {
    spacing: f32,
    children: Vec<Positioner<Proxy<W>>>,
    size: Size,
    alignment: Alignment,
    orientation: Orientation,
}

impl<W> FromIterator<W> for SimpleLayout<W> {
    fn from_iter<T: IntoIterator<Item = W>>(iter: T) -> Self {
        let mut layout = SimpleLayout::new();
        for widget in iter {
            layout.children.push(child(widget));
        }
        layout
    }
}

impl<D, W> Container<D, W> for SimpleLayout<W>
where
    W: Widget<D>,
{
    fn len(&self) -> usize {
        self.children.len()
    }
    fn add(&mut self, widget: W) {
        self.children.push(child(widget));
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

impl<W: Geometry> Geometry for SimpleLayout<W> {
    fn width(&self) -> f32 {
        self.size.width
    }
    fn height(&self) -> f32 {
        self.size.height
    }
}

impl<D> Default for SimpleLayout<Box<dyn Widget<D>>> {
    fn default() -> Self {
        SimpleLayout {
            spacing: 0.,
            size: Size::default(),
            children: Vec::new(),
            alignment: Alignment::Start,
            orientation: Orientation::Horizontal,
        }
    }
}

impl<D> SimpleLayout<Box<dyn Widget<D>>> {
    /// The default behaviour.
    pub fn add<W: Widget<D> + 'static>(&mut self, widget: W) {
        self.children.push(child(Box::new(widget)));
    }
}

impl<W> SimpleLayout<W> {
    pub fn new() -> Self {
        SimpleLayout {
            spacing: 0.,
            size: Size::default(),
            children: Vec::new(),
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
        self.children.clear();
    }
    pub fn inner(&mut self) -> &mut [Positioner<Proxy<W>>] {
        self.children.as_mut_slice()
    }
}

impl<D, W: Widget<D>> Widget<D> for SimpleLayout<W> {
    fn draw_scene(&mut self, mut scene: Scene) {
        for widget in self.children.iter_mut() {
            match scene.next() {
                Some(scene) => {
                    widget.draw_scene(scene);
                }
                None => continue
            }
            if let Some(scene) = scene.append_node(RenderNode::None, self.size) {
                widget.draw_scene(scene);
            }
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
        let (mut dx, mut dy) = (0., 0.);
        self.size = match self.orientation {
            Orientation::Vertical => self
                .children
                .iter_mut()
                .map(|widget| {
                    widget.set_coords(dx, dy);
                    let (inner_width, inner_height) = widget
                        .layout(ctx, &constraints.with_max(constraints.maximum_width(), 0.))
                        .into();
                    dy += inner_height + self.spacing;
                    Size::new(inner_width, inner_height)
                })
                .reduce(|accum, size| {
                    Size::new(
                        accum.width.max(size.width),
                        accum.height + size.height + self.spacing,
                    )
                })
                .unwrap_or_default(),
            Orientation::Horizontal => self
                .children
                .iter_mut()
                .map(|widget| {
                    widget.set_coords(dx, dy);
                    let (inner_width, inner_height) = widget
                        .layout(ctx, &constraints.with_max(0., constraints.maximum_height()))
                        .into();
                    dx += inner_width + self.spacing;
                    Size::new(inner_width, inner_height)
                })
                .reduce(|accum, size| {
                    Size::new(
                        accum.width + size.width + self.spacing,
                        accum.height.max(size.height),
                    )
                })
                .unwrap_or_default(),
        };
        self.size
    }
}
