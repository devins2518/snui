pub mod center_box;
pub mod layout_box;
pub mod widget_layout;

use crate::*;
pub use center_box::CenterBox;
pub use layout_box::LayoutBox;
use scene::Coords;
pub use widget_layout::WidgetLayout;

pub trait Container<R: 'static>: Geometry + FromIterator<Child<R>> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn remove(&mut self, index: usize) -> Child<R>;
    fn add(&mut self, widget: impl Widget<R> + 'static);
    fn pop(&mut self) -> Child<R> {
        self.remove(self.len() - 1)
    }
}

pub fn apply_width<R, W: Widget<R>>(
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

pub fn apply_height<R, W: Widget<R>>(
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

pub struct Child<R> {
    coords: Coords,
    damage: Damage,
    queue_draw: bool,
    widget: Box<dyn Widget<R>>,
}

impl<R> Child<R> {
    pub(crate) fn new(widget: impl Widget<R> + 'static) -> Self {
        Child {
            queue_draw: false,
            damage: Damage::None,
            coords: Coords::new(0., 0.),
            widget: Box::new(widget),
        }
    }
    fn create_node_ext(&mut self, x: f32, y: f32, width: f32, height: f32) -> RenderNode {
        let node = self.create_node(x, y);
        if !node.is_none() {
            return RenderNode::Extension {
                background: scene::Instruction::empty(
                    x,
                    y,
                    width,
                    height,
                ),
                border: None,
                node: Box::new(node),
            };
        }
        node
    }
}

impl<R> Geometry for Child<R> {
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
}

impl<R> From<Box<dyn Widget<R>>> for Child<R> {
    fn from(widget: Box<dyn Widget<R>>) -> Self {
        Child {
            queue_draw: false,
            damage: Damage::None,
            coords: Coords::new(0., 0.),
            widget,
        }
    }
}

impl<R> Widget<R> for Child<R> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        if self.queue_draw || self.damage.is_some() {
            self.damage = Damage::None;
            return self
                .widget
                .create_node(x + self.coords.x, y + self.coords.y);
        }
        RenderNode::None
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<R>, event: &Event<R>) -> Damage {
        self.damage = self.damage.max(match event {
            Event::Pointer(mut x, mut y, p) => {
                x -= self.coords.x;
                y -= self.coords.y;
                let result = self.widget.sync(ctx, &Event::Pointer(x, y, *p));
                result
            }
            Event::Frame => self.widget.sync(ctx, event),
            _ => self.widget.sync(ctx, event),
        });
        self.queue_draw = self.damage.is_some() || event.is_frame();
        self.damage
    }
    fn contains(&self, x: f32, y: f32) -> bool {
        self.widget.contains(x + self.coords.x, y + self.coords.y)
    }
}
