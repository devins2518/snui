use crate::*;

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
    bound: f32,
    size: Size,
    orientation: Orientation,
    widget: Positioner<Proxy<W>>,
}

impl<W> ScrollBox<W> {
    pub fn new(widget: W) -> Self {
        Self {
            size: Size::default(),
            widget: child(widget),
            orientation: Orientation::Vertical,
            bound: 100.,
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

impl<W> Scrollable for ScrollBox<W> {
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
                    coords.x = (coords.x - delta).max((self.bound - self.size.width).min(0.))
                }
                None => coords.x = (coords.x - 10.).min((self.bound - self.size.width).min(0.)),
            },
            Orientation::Vertical => match step {
                Some(delta) => {
                    coords.y = (coords.y - delta).max((self.bound - self.size.height).min(0.))
                }
                None => coords.y = (coords.y - 10.).min((self.bound - self.size.height).min(0.)),
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
        self.size.width
    }
    fn inner_height(&self) -> f32 {
        self.size.width
    }
    fn orientation(&self) -> Orientation {
        self.orientation
    }
}

impl<W> Geometry for ScrollBox<W> {
    fn width(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => self.bound,
            _ => self.size.width,
        }
    }
    fn height(&self) -> f32 {
        match self.orientation {
            Orientation::Vertical => self.bound,
            _ => self.size.height,
        }
    }
}

impl<T, W> Widget<T> for ScrollBox<W>
where
    W: Widget<T>,
{
    fn draw_scene(&mut self, mut scene: Scene) {
        if let Some(scene) = scene.apply_clip(Size::new(self.width(), self.height())) {
            self.widget.draw_scene(scene)
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<T>, event: Event<'d>) -> Damage {
        match event {
            Event::Pointer(_, _, Pointer::Scroll { orientation, step }) => {
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
                }
                self.widget
                    .sync(ctx, event)
                    .max(self.widget.sync(ctx, Event::Draw))
            }
            _ => self.widget.sync(ctx, event),
        }
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        self.size = self.widget.layout(ctx, constraints);
        match self.orientation {
            Orientation::Horizontal => {
                self.bound = constraints.maximum_width();
            }
            Orientation::Vertical => {
                self.bound = constraints.maximum_height();
            }
        }
        (self.width(), self.height()).into()
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
