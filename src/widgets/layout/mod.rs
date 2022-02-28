pub mod flex;

use crate::*;
use scene::Coords;
use std::ops::{Deref, DerefMut};
use widgets::Style;

/// Widgets which contain one or more widgets
pub trait Container<W> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn remove(&mut self, index: usize) -> W;
    fn add(&mut self, widget: W);
    fn pop(&mut self) -> W {
        self.remove(self.len() - 1)
    }
    fn children(&mut self) -> Vec<&mut W>;
}

/// Widget with relative positioning
#[derive(Clone, Debug, PartialEq)]
pub struct Positioner<W> {
    coords: Coords,
    old_coords: Coords,
    pub(crate) widget: W,
}

impl<W> Positioner<W> {
    pub(crate) fn new(widget: W) -> Self {
        Positioner {
            widget,
            old_coords: Coords::new(0., 0.),
            coords: Coords::new(0., 0.),
        }
    }
    pub fn translate(mut self, x: f32, y: f32) -> Self {
        self.coords = Coords::new(self.coords.x + x, self.coords.y + y);
        self
    }
    pub fn swap(&mut self, coords: Coords) {
        self.old_coords = self.coords;
        self.coords = coords;
    }
    pub fn set_coords(&mut self, x: f32, y: f32) {
        self.old_coords = self.coords;
        self.coords = Coords::new(x, y);
    }
    pub fn coords(&self) -> Coords {
        self.coords
    }
}

impl<W: Geometry> Geometry for Positioner<W> {
    fn width(&self) -> f32 {
        self.widget.width()
    }
    fn height(&self) -> f32 {
        self.widget.height()
    }
    fn contains(&self, position: &scene::Coords) -> bool {
        self.widget
            .contains(&position.translate(self.coords.x, self.coords.y))
    }
}

impl<T, W: Widget<T>> Widget<T> for Positioner<W> {
    fn draw_scene(&mut self, scene: Scene) {
        self.widget
            .draw_scene(scene.translate(self.coords.x, self.coords.y))
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<T>, event: Event) -> Damage {
        let damage = match event {
            Event::Pointer(MouseEvent { position, pointer }) => self.widget.sync(
                ctx,
                MouseEvent::new(position.translate(-self.coords.x, -self.coords.y), pointer),
            ),
            _ => self.widget.sync(ctx, event),
        };
        self.old_coords
            .ne(&self.coords)
            .then(|| {
                self.old_coords = self.coords;
                self.widget.sync(ctx, Event::Draw).max(damage)
            })
            .unwrap_or(damage)
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> crate::Size {
        self.widget.layout(ctx, constraints)
    }
}

impl<W: Style> Style for Positioner<W> {
    fn set_texture<B: Into<scene::Texture>>(&mut self, texture: B) {
        self.widget.set_texture(texture)
    }
    fn set_top_left_radius(&mut self, radius: f32) {
        self.widget.set_top_left_radius(radius);
    }
    fn set_top_right_radius(&mut self, radius: f32) {
        self.widget.set_top_right_radius(radius);
    }
    fn set_bottom_right_radius(&mut self, radius: f32) {
        self.widget.set_bottom_right_radius(radius);
    }
    fn set_bottom_left_radius(&mut self, radius: f32) {
        self.widget.set_bottom_left_radius(radius);
    }
}

use widgets::scroll::Scrollable;

impl<W: Scrollable> Scrollable for Positioner<W> {
    fn forward(&mut self, step: Option<f32>) {
        self.widget.forward(step)
    }
    fn backward(&mut self, step: Option<f32>) {
        self.widget.backward(step)
    }
    fn inner_height(&self) -> f32 {
        self.widget.inner_height()
    }
    fn inner_width(&self) -> f32 {
        self.widget.inner_width()
    }
    fn orientation(&self) -> Orientation {
        self.widget.orientation()
    }
    fn position(&self) -> f32 {
        self.widget.position()
    }
}

impl<W> Deref for Positioner<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<W> DerefMut for Positioner<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}

pub fn child<W>(widget: W) -> Positioner<Proxy<W>> {
    Positioner::new(Proxy::new(widget))
}
