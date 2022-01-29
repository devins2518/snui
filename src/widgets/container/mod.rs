pub mod center_box;
pub mod layout_box;
pub mod widget_layout;

use crate::*;
use scene::Coords;
use widgets::Style;
pub use center_box::CenterBox;
pub use layout_box::LayoutBox;
pub use widget_layout::WidgetLayout;

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

pub fn apply_width<W: Geometry>(
    widgets: &mut [W],
    fixed: &mut Vec<usize>,
    index: usize,
    width: f32,
) {
    match fixed.binary_search(&index) {
        Ok(index) => {
            if index > 0 {
                apply_width(widgets, fixed, index - 1, width);
            }
        }
        Err(pos) => {
            if let Err(w) = widgets[index].set_width(width) {
                fixed.insert(pos, index);
                let delta = width - w;
                if index > 0 {
                    apply_width(widgets, fixed, index - 1, width + delta);
                }
            }
        }
    }
}

pub fn apply_height<W: Geometry>(
    widgets: &mut [W],
    fixed: &mut Vec<usize>,
    index: usize,
    height: f32,
) {
    match fixed.binary_search(&index) {
        Ok(index) => {
            if index > 0 {
                apply_height(widgets, fixed, index - 1, height);
            }
        }
        Err(pos) => {
            if let Err(w) = widgets[index].set_height(height) {
                fixed.insert(pos, index);
                let delta = height - w;
                if index > 0 {
                    apply_height(widgets, fixed, index - 1, height + delta);
                }
            }
        }
    }
}

pub struct Positioner<W> {
    coords: Coords,
    pub(crate) widget: W,
}

impl<W> Positioner<W> {
    pub(crate) fn new(widget: W) -> Self {
        Positioner {
            widget,
            coords: Coords::new(0., 0.),
        }
    }
    pub fn set_coords(&mut self, x: f32, y: f32) {
        self.coords = Coords::new(x, y);
    }
}

impl<W: Geometry> Geometry for Positioner<W> {
    fn width(&self) -> f32 {
        self.widget.width()
    }
    fn height(&self) -> f32 {
        self.widget.height()
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.widget.set_width(width)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.widget.set_height(height)
    }
    fn contains(&self, x: f32, y: f32) -> bool {
        self.widget.contains(x + self.coords.x, y + self.coords.y)
    }
}

impl<D, W: Widget<D>> Widget<D> for Positioner<W> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        return self
            .widget
            .create_node(transform.pre_translate(self.coords.x, self.coords.y));
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event) -> Damage {
        match event {
            Event::Pointer(mut x, mut y, p) => {
                x -= self.coords.x;
                y -= self.coords.y;
                self.widget.sync(ctx, Event::Pointer(x, y, p))
            }
            _ => self.widget.sync(ctx, event),
        }
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
    fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.widget.set_radius(tl, tr, br, bl)
    }
}

pub fn child<W>(widget: W) -> Positioner<Proxy<W>> {
    Positioner::new(Proxy::new(widget))
}
