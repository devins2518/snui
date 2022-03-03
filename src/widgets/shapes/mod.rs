//! Simple shapes for building widgets

pub mod rectangle;
pub mod style;
use crate::scene::*;

pub use rectangle::{BorderedRectangle, Rectangle};
pub use style::WidgetStyle;

// A trait used to theme widgets.
pub trait Style: Sized {
    fn set_texture<B: Into<Texture>>(&mut self, texture: B);
    fn set_top_left_radius(&mut self, radius: f32);
    fn set_top_right_radius(&mut self, radius: f32);
    fn set_bottom_right_radius(&mut self, radius: f32);
    fn set_bottom_left_radius(&mut self, radius: f32);
    fn set_radius(&mut self, radius: f32) {
        self.set_top_left_radius(radius);
        self.set_top_right_radius(radius);
        self.set_bottom_right_radius(radius);
        self.set_bottom_left_radius(radius);
    }
    fn radius(mut self, radius: f32) -> Self {
        self.set_radius(radius);
        self
    }
    fn top_left_radius(mut self, radius: f32) -> Self {
        self.set_top_left_radius(radius);
        self
    }
    fn top_right_radius(mut self, radius: f32) -> Self {
        self.set_top_right_radius(radius);
        self
    }
    fn bottom_right_radius(mut self, radius: f32) -> Self {
        self.set_bottom_right_radius(radius);
        self
    }
    fn bottom_left_radius(mut self, radius: f32) -> Self {
        self.set_bottom_left_radius(radius);
        self
    }
    fn texture<B: Into<Texture>>(mut self, texture: B) -> Self {
        self.set_texture(texture);
        self
    }
}
