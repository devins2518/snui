pub mod center_box;
pub mod layout_box;
pub mod widget_layout;

use crate::*;
use scene::Coords;
pub use center_box::Centerbox;
pub use layout_box::LayoutBox;
pub use widget_layout::WidgetLayout;

pub struct Child {
    coords: Coords,
    widget: Box<dyn Widget>,
}

impl Child {
    fn new(widget: impl Widget + 'static) -> Self {
        Child {
            coords: Coords::new(0., 0.),
            widget: Box::new(widget),
        }
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

impl Widget for Child {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        self.widget.create_node(x + self.coords.x, y + self.coords.y)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        if let Event::Pointer(mut x, mut y, p) = event {
            x -= self.coords.x;
            y -= self.coords.y;
            self.widget.sync(ctx, Event::Pointer(x, y, p))
        } else {
            self.widget.sync(ctx, event)
        }
    }
    fn contains(&self, x: f32, y: f32) -> bool {
        self.widget.contains(
            x + self.coords.x,
            y + self.coords.y,
        )
    }
}

// Write macro similar to vec!
// #[macro_export]
// macro_rules! add {
//     () => ()
//     ( $() )
// }
