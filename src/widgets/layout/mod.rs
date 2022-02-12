pub mod dynamic;
pub mod simple;

use crate::*;
use scene::Coords;
use std::ops::{Deref, DerefMut};
use widgets::Style;

/// Widgets which contain one or more widgets
pub trait Container<D, W>: Geometry
where
    W: Widget<D>,
{
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn remove(&mut self, index: usize) -> W;
    fn add(&mut self, widget: W);
    fn pop(&mut self) -> W {
        self.remove(self.len() - 1)
    }
    fn widgets(&mut self) -> Vec<&mut W>;
}

#[derive(Clone, Debug, PartialEq, Copy)]
enum Size {
    Set(f32),
    Var(f32),
}

impl From<Size> for f32 {
    fn from(s: Size) -> f32 {
        match s {
            Size::Set(f) => f,
            Size::Var(f) => f,
        }
    }
}

/// Takes a slice of widgets and does its best to ensure the layout respect the width contrain.
pub fn apply_width<W: Geometry>(widgets: &mut [W], width: f32) -> f32 {
    let mut delta: f32;
    let mut c_width;
    let mut extra = 0.;
    let mut layout = widgets
        .iter()
        .map(|widget| {
            (
                widget.minimum_width(),
                Size::Var(widget.minimum_width()),
                widget.maximum_width(),
            )
        })
        .collect::<Vec<(f32, Size, f32)>>();
    let len = widgets.len();
    let mut count = widgets.len();
    let mut iter = (0..widgets.len()).cycle();
    while {
        c_width = layout.iter().map(|(_, s, _)| f32::from(*s)).sum();
        delta = width - c_width;
        delta > 0. && count > 0
    } {
        if let Some(i) = iter.next() {
            let (min, size, max) = layout[i];
            match size {
                Size::Var(size) => {
                    let u_width = (delta / (len - i) as f32) + size + extra;
                    let size = (u_width).clamp(min, max).round();
                    if u_width >= max {
                        layout[i].1 = Size::Set(size);
                        count -= 1;
                        extra = 0.;
                    } else {
                        layout[i].1 = Size::Var(size);
                        extra = u_width - size;
                    }
                }
                Size::Set(_) => {}
            }
        }
    }
    for (i, (_, width, _)) in layout.into_iter().enumerate() {
        let _ = widgets[i].set_width(width.into());
    }
    c_width
}

/// Takes a slice of widgets and does its best to ensure the layout respect the height contrain.
pub fn apply_height<W: Geometry>(widgets: &mut [W], height: f32) -> f32 {
    let mut delta: f32;
    let mut c_height;
    let mut extra = 0.;
    let mut layout = widgets
        .iter()
        .map(|widget| {
            (
                widget.minimum_height(),
                Size::Var(widget.minimum_height()),
                widget.maximum_height(),
            )
        })
        .collect::<Vec<(f32, Size, f32)>>();
    let len = widgets.len();
    let mut count = len;
    let mut iter = (0..len).cycle();
    while {
        c_height = layout.iter().map(|(_, s, _)| f32::from(*s)).sum();
        delta = height - c_height;
        delta > 0. && count > 0
    } {
        if let Some(i) = iter.next() {
            let (min, size, max) = layout[i];
            match size {
                Size::Var(size) => {
                    let u_height = (delta / (len - i) as f32) + size + extra;
                    let size = (u_height).clamp(min, max).round();
                    if u_height >= max {
                        layout[i].1 = Size::Set(size);
                        count -= 1;
                        extra = 0.;
                    } else {
                        layout[i].1 = Size::Var(size);
                        extra = u_height - size;
                    }
                }
                Size::Set(_) => {}
            }
        }
    }
    for (i, (_, height, _)) in layout.into_iter().enumerate() {
        let _ = widgets[i].set_height(height.into());
    }
    c_height
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
    fn set_width(&mut self, width: f32) {
        self.widget.set_width(width)
    }
    fn set_height(&mut self, height: f32) {
        self.widget.set_height(height)
    }
    fn contains(&self, x: f32, y: f32) -> bool {
        self.widget.contains(x + self.coords.x, y + self.coords.y)
    }
    fn maximum_height(&self) -> f32 {
        self.widget.maximum_height()
    }
    fn minimum_height(&self) -> f32 {
        self.widget.minimum_height()
    }
    fn maximum_width(&self) -> f32 {
        self.widget.maximum_width()
    }
    fn minimum_width(&self) -> f32 {
        self.widget.minimum_width()
    }
}

impl<D, W: Widget<D>> Widget<D> for Positioner<W> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        return self
            .widget
            .create_node(transform.pre_translate(self.coords.x, self.coords.y));
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event) -> Damage {
        let damage = match event {
            Event::Pointer(mut x, mut y, p) => {
                x -= self.coords.x;
                y -= self.coords.y;
                self.widget.sync(ctx, Event::Pointer(x, y, p))
            }
            _ => self.widget.sync(ctx, event),
        };
        self.old_coords
            .ne(&self.coords)
            .then(|| damage.max(Damage::Partial))
            .unwrap_or(damage)
    }
    fn prepare_draw(&mut self) {
        self.widget.prepare_draw()
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> (f32, f32) {
        self.widget
            .layout(ctx, &constraints.crop(self.coords.x, self.coords.y))
    }
}

impl<W: Style> Style for Positioner<W> {
    fn set_background<B: Into<scene::Texture>>(&mut self, texture: B) {
        self.widget.set_background(texture)
    }
    fn set_border_size(&mut self, size: f32) {
        self.widget.set_border_size(size)
    }
    fn set_border_texture<T: Into<scene::Texture>>(&mut self, texture: T) {
        self.widget.set_border_texture(texture)
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
