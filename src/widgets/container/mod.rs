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
        if width > self.width() {
            return self.widget.set_width(width);
        }
        Ok(())
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        if height > self.height() {
            return self.widget.set_height(height);
        }
        Ok(())
    }
}

// Write macro similar to vec!
