//! A layout widget that supports interactive resize.

use crate::widgets::layout::*;
use crate::widgets::Spacer;
use crate::*;
use scene::Scene;

pub struct Flex<W> {
    size: Size,
    orientation: Orientation,
    children: Vec<Positioner<Proxy<W>>>,
}

impl<W> Flex<W> {
    pub fn row() -> Self {
        Self {
            size: Size::default(),
            children: Vec::new(),
            orientation: Orientation::Horizontal,
        }
    }
    pub fn column() -> Self {
        Self {
            size: Size::default(),
            children: Vec::new(),
            orientation: Orientation::Vertical,
        }
    }
    pub fn with(mut self, widget: W) -> Self {
        self.add(widget);
        self
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

impl<W> FromIterator<W> for Flex<W> {
    fn from_iter<T: IntoIterator<Item = W>>(iter: T) -> Self {
        let mut this = Flex::row();
        for widget in iter {
            this.children.push(child(widget));
        }
        this
    }
}

impl<W> Container<W> for Flex<W> {
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

impl<T, W: Widget<T>> Widget<T> for Flex<W> {
    fn draw_scene(&mut self, mut scene: Scene) {
        for widget in self.children.iter_mut() {
            if let Some(scene) = scene.next(self.size) {
                widget.draw_scene(scene);
            }
        }
        scene.truncate(self.len())
    }
    fn event<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
        self.children
            .iter_mut()
            .map(|widget| widget.event(ctx, event))
            .max()
            .unwrap_or_default()
    }
    fn update<'s>(&'s mut self, ctx: &mut SyncContext<T>) -> Damage {
        self.children
            .iter_mut()
            .map(|widget| widget.update(ctx))
            .max()
            .unwrap_or_default()
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        let len = self.len();
        self.size = match self.orientation {
            Orientation::Vertical => {
                let mut dy = 0.;
                self.children
                    .iter_mut()
                    .enumerate()
                    .map(|(i, widget)| {
                        widget.set_coords(0., dy);
                        let height = (constraints.maximum_height() - dy) / (len - i) as f32;
                        let Size { width, height } = widget.layout(
                            ctx,
                            &constraints.with_max(
                                constraints.maximum_width(),
                                height.min(constraints.maximum_height()),
                            ),
                        );
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
                    .map(|(i, widget)| {
                        let width = (constraints.maximum_width() - dx) / (len - i) as f32;
                        widget.set_coords(dx, 0.);
                        let Size { width, height } = widget.layout(
                            ctx,
                            &constraints.with_max(
                                width.min(constraints.maximum_width()),
                                constraints.maximum_height(),
                            ),
                        );
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

impl<T> Flex<Box<dyn Widget<T>>> {
    pub fn add_child<W: Widget<T> + 'static>(&mut self, widget: W) {
        self.children.push(child(Box::new(widget)));
    }
    pub fn with_child<W: Widget<T> + 'static>(mut self, widget: W) -> Self {
        self.children
            .push(Positioner::new(Proxy::new(Box::new(widget))));
        self
    }
    pub fn with_spacing(self, size: f32) -> Self {
        match self.orientation {
            Orientation::Vertical => self.with_child(Spacer::height(size)),
            Orientation::Horizontal => self.with_child(Spacer::width(size)),
        }
    }
}
