pub mod center_box;
pub mod layout_box;
pub mod widget_layout;

use crate::*;
pub use center_box::CenterBox;
pub use layout_box::LayoutBox;
use scene::Coords;
pub use widget_layout::WidgetLayout;

pub trait Container: Geometry + FromIterator<Child> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn remove(&mut self, index: usize) -> Child;
    fn add(&mut self, widget: impl Widget + 'static);
    fn pop(&mut self) -> Child {
        self.remove(self.len() - 1)
    }
}

pub struct Child {
    coords: Coords,
    damage: Damage,
    queue_draw: bool,
    widget: Box<dyn Widget>,
}

impl Child {
    pub(crate) fn new(widget: impl Widget + 'static) -> Self {
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
                    x + self.coords.x,
                    y + self.coords.y,
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

impl Geometry for Child {
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

impl From<Box<dyn Widget>> for Child {
    fn from(widget: Box<dyn Widget>) -> Self {
        Child {
            queue_draw: false,
            damage: Damage::None,
            coords: Coords::new(0., 0.),
            widget,
        }
    }
}

impl Widget for Child {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        if self.queue_draw || self.damage.is_some() {
            self.damage = Damage::None;
            return self
                .widget
                .create_node(x + self.coords.x, y + self.coords.y);
        }
        RenderNode::None
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) -> Damage {
        self.damage = self.damage.max(match event {
            Event::Pointer(mut x, mut y, p) => {
                x -= self.coords.x;
                y -= self.coords.y;
                let result = self.widget.sync(ctx, Event::Pointer(x, y, p));
                result
            }
            Event::Frame => self.widget.sync(ctx, event),
            _ => self.widget.sync(ctx, event),
        });
        self.queue_draw = self.damage.is_some() || event == Event::Frame;
        self.damage
    }
    fn contains(&self, x: f32, y: f32) -> bool {
        self.widget.contains(x + self.coords.x, y + self.coords.y)
    }
}
