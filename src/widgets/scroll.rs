use crate::*;
use scene::Region;
use std::ops::{Deref, DerefMut};
use widgets::layout::child;
use widgets::*;

/// For widgets that move linearly within in a bounded region.
pub trait Scrollable {
    fn forward(&mut self, step: Option<f32>);
    fn backward(&mut self, step: Option<f32>);
    fn orientation(&self) -> Orientation;
    fn inner_width(&self) -> f32;
    fn inner_height(&self) -> f32;
    fn position(&self) -> f32;
}

pub struct ScrollBox<W> {
    size: f32,
    orientation: Orientation,
    widget: Positioner<Proxy<W>>,
}

impl<W> ScrollBox<W> {
    pub fn new(widget: W) -> Self {
        Self {
            widget: child(widget),
            orientation: Orientation::Vertical,
            size: 100.,
        }
    }
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.set_orientation(orientation);
        self
    }
}

impl<W: Geometry> Scrollable for ScrollBox<W> {
    fn forward(&mut self, step: Option<f32>) {
        let mut coords = self.widget.coords();
        match self.orientation {
            Orientation::Horizontal => match step {
                Some(delta) => coords.x = (coords.x + delta).min(0.),
                None => coords.x = (coords.x + 10.).min(0.),
            },
            Orientation::Vertical => match step {
                Some(delta) => coords.y = (coords.y + delta).min(0.),
                None => coords.y = (coords.y + 10.).min(0.),
            },
        }
        self.widget.swap(coords);
    }
    fn backward(&mut self, step: Option<f32>) {
        let mut coords = self.widget.coords();
        match self.orientation {
            Orientation::Horizontal => match step {
                Some(delta) => {
                    coords.x = (coords.x - delta).max((self.size - self.inner_width()).min(0.))
                }
                None => coords.x = (coords.x - 10.).min((self.size - self.inner_width()).min(0.)),
            },
            Orientation::Vertical => match step {
                Some(delta) => {
                    coords.y = (coords.y - delta).max((self.size - self.inner_height()).min(0.))
                }
                None => coords.y = (coords.y - 10.).min((self.size - self.inner_height()).min(0.)),
            },
        }
        self.widget.swap(coords);
    }
    fn position(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => self.widget.coords().x,
            Orientation::Vertical => self.widget.coords().y,
        }
    }
    fn inner_width(&self) -> f32 {
        self.widget.width()
    }
    fn inner_height(&self) -> f32 {
        self.widget.height()
    }
    fn orientation(&self) -> Orientation {
        self.orientation
    }
}

impl<W: Geometry> GeometryExt for ScrollBox<W> {
    fn apply_width(&mut self, width: f32) {
        match self.orientation {
            Orientation::Horizontal => self.size = width,
            _ => {
                self.widget.set_width(width);
            }
        }
    }
    fn apply_height(&mut self, height: f32) {
        match self.orientation {
            Orientation::Vertical => self.size = height,
            _ => {
                self.widget.set_height(height);
            }
        }
    }
}

impl<W: Geometry> Geometry for ScrollBox<W> {
    fn width(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => self.size,
            _ => self.widget.width(),
        }
    }
    fn height(&self) -> f32 {
        match self.orientation {
            Orientation::Vertical => self.size,
            _ => self.widget.height(),
        }
    }
    fn set_width(&mut self, width: f32) {
        let c_width = width.clamp(self.minimum_width(), self.maximum_width());
        match self.orientation {
            Orientation::Horizontal => {
                self.size = c_width;
            }
            Orientation::Vertical => self.widget.set_width(c_width),
        }
    }
    fn set_height(&mut self, height: f32) {
        let c_height = height.clamp(self.minimum_height(), self.maximum_height());
        match self.orientation {
            Orientation::Vertical => {
                self.size = c_height;
            }
            Orientation::Horizontal => self.widget.set_height(c_height),
        }
    }
    fn maximum_height(&self) -> f32 {
        match self.orientation {
            Orientation::Vertical => std::f32::INFINITY,
            _ => self.widget.maximum_height(),
        }
    }
    fn minimum_height(&self) -> f32 {
        match self.orientation {
            Orientation::Vertical => 0.,
            _ => self.widget.minimum_height(),
        }
    }
    fn maximum_width(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => std::f32::INFINITY,
            _ => self.widget.maximum_width(),
        }
    }
    fn minimum_width(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => 0.,
            _ => self.widget.minimum_width(),
        }
    }
}

impl<D, W> Widget<D> for ScrollBox<W>
where
    W: Widget<D>,
{
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        if let Some(node) = self.widget.create_node(transform).as_option() {
            let region = Region::from_transform(transform, self.width(), self.height());
            RenderNode::Clip(region.into(), Box::new(node))
        } else {
            RenderNode::None
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        match event {
            Event::Pointer(_, _, p) => match p {
                Pointer::Scroll { orientation, step } => {
                    let coords = self.widget.coords();
                    if orientation == self.orientation {
                        match step {
                            Step::Increment(i) => {
                                if i.is_positive() {
                                    for _ in 0..i {
                                        self.forward(None);
                                    }
                                } else {
                                    for _ in i..0 {
                                        self.backward(None);
                                    }
                                }
                            }
                            Step::Value(value) => {
                                if value.is_sign_positive() {
                                    self.forward(Some(value.abs()));
                                } else {
                                    self.backward(Some(value.abs()));
                                }
                            }
                        }
                        if coords != self.widget.coords() {
                            self.prepare_draw();
                        }
                    }
                    self.widget.sync(ctx, event)
                }
                _ => self.widget.sync(ctx, event),
            },
            _ => self.widget.sync(ctx, event),
        }
    }
    fn prepare_draw(&mut self) {
        self.widget.prepare_draw()
    }
    fn layout(&mut self, ctx: &mut LayoutCtx) -> (f32, f32) {
        let (width, height) = self.widget.layout(ctx);
        match self.orientation {
            Orientation::Horizontal => (self.size, height),
            Orientation::Vertical => (width, self.size),
        }
    }
}

impl<W> Deref for ScrollBox<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        self.widget.deref().deref()
    }
}

impl<W> DerefMut for ScrollBox<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.widget.deref_mut().deref_mut()
    }
}

impl<W: Style> Style for ScrollBox<W> {
    fn set_background<B: Into<scene::Texture>>(&mut self, texture: B) {
        self.widget.set_background(texture)
    }
    fn set_bottom_left_radius(&mut self, radius: f32) {
        self.widget.set_bottom_left_radius(radius)
    }
    fn set_top_right_radius(&mut self, radius: f32) {
        self.widget.set_top_right_radius(radius)
    }
    fn set_top_left_radius(&mut self, radius: f32) {
        self.widget.set_top_left_radius(radius)
    }
    fn set_bottom_right_radius(&mut self, radius: f32) {
        self.widget.set_bottom_right_radius(radius)
    }
    fn set_border_size(&mut self, size: f32) {
        self.widget.set_border_size(size)
    }
    fn set_border_texture<T: Into<scene::Texture>>(&mut self, texture: T) {
        self.widget.set_border_texture(texture)
    }
}
