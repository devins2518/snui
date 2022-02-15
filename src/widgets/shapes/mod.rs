pub mod rectangle;
pub mod style;
use crate::scene::*;

pub use rectangle::Rectangle;
pub use style::WidgetStyle;

pub trait Style: Sized {
    fn set_background<B: Into<Texture>>(&mut self, texture: B);
    // fn set_border_size(&mut self, size: f32);
    // fn set_border_texture<T: Into<Texture>>(&mut self, texture: T);
    // fn set_border<T: Into<Texture>>(&mut self, texture: T, size: f32) {
    // self.set_border_texture(texture);
    // self.set_border_size(size);
    // }
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
    fn background<B: Into<Texture>>(mut self, texture: B) -> Self {
        self.set_background(texture);
        self
    }
    // fn border_size(mut self, size: f32) -> Self {
    //     self.set_border_size(size);
    //     self
    // }
    // fn border_texture<T: Into<Texture>>(mut self, texture: T) -> Self {
    //     self.set_border_texture(texture);
    //     self
    // }
    // fn border<T: Into<Texture>>(mut self, texture: T, size: f32) -> Self {
    //     self.set_border(texture, size);
    //     self
    // }
}
