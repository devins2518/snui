pub mod center_box;
pub mod layout_box;
pub mod widget_layout;

use crate::*;
pub use center_box::CenterBox;
pub use layout_box::LayoutBox;
use scene::Coords;
pub use widget_layout::WidgetLayout;

pub trait Container<M: 'static>: Geometry + FromIterator<Child<M>> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn remove(&mut self, index: usize) -> Child<M>;
    fn add(&mut self, widget: impl Widget<M> + 'static);
    fn pop(&mut self) -> Child<M> {
        self.remove(self.len() - 1)
    }
}

pub fn apply_width<M, W: Widget<M>>(
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

pub fn apply_height<M, W: Widget<M>>(
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

pub struct Child<M> {
    coords: Coords,
    damage: Damage,
    widget: Box<dyn Widget<M>>,
}

impl<M> Child<M> {
    pub(crate) fn new(widget: impl Widget<M> + 'static) -> Self {
        Child {
            damage: Damage::None,
            coords: Coords::new(0., 0.),
            widget: Box::new(widget),
        }
    }
    pub fn set_coords(&mut self, x: f32, y: f32) {
        self.coords = Coords::new(x, y);
    }
}

impl<M> Geometry for Child<M> {
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

impl<M> From<Box<dyn Widget<M>>> for Child<M> {
    fn from(widget: Box<dyn Widget<M>>) -> Self {
        Child {
            damage: Damage::None,
            coords: Coords::new(0., 0.),
            widget,
        }
    }
}

impl<M> Widget<M> for Child<M> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        if self.damage.is_some() {
            self.damage = Damage::None;
            return self
                .widget
                .create_node(x + self.coords.x, y + self.coords.y);
        }
        RenderNode::None
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<M>) -> Damage {
        self.damage = self.damage.max(match event {
            Event::Pointer(mut x, mut y, p) => {
                x -= self.coords.x;
                y -= self.coords.y;
                let result = self.widget.sync(ctx, Event::Pointer(x, y, p));
                result
            }
            Event::Configure(_) | Event::Prepare => {
                Damage::Partial.max(self.widget.sync(ctx, event))
            }
            _ => self.widget.sync(ctx, event),
        });
        self.damage
    }
}
