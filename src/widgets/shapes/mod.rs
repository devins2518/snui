pub mod rectangle;
pub mod style;
use crate::scene::*;

pub use rectangle::Rectangle;
pub use style::WidgetStyle;

pub trait Style: Sized {
    fn set_background<B: Into<Texture>>(&mut self, texture: B);
    fn set_border_size(&mut self, size: f32);
    fn set_border_texture<T: Into<Texture>>(&mut self, texture: T);
    fn set_border<T: Into<Texture>>(&mut self, texture: T, size: f32) {
        self.set_border_texture(texture);
        self.set_border_size(size);
    }
    fn set_radius_top_left(&mut self, radius: f32);
    fn set_radius_top_right(&mut self, radius: f32);
    fn set_radius_bottom_right(&mut self, radius: f32);
    fn set_radius_bottom_left(&mut self, radius: f32);
    fn set_radius(&mut self, radius: f32) {
        self.set_radius_top_left(radius);
        self.set_radius_top_right(radius);
        self.set_radius_bottom_right(radius);
        self.set_radius_bottom_left(radius);
    }
    fn radius(mut self, radius: f32) -> Self {
        self.set_radius(radius);
        self
    }
    fn radius_top_left(mut self, radius: f32) -> Self {
        self.set_radius_top_left(radius);
        self
    }
    fn radius_top_right(mut self, radius: f32) -> Self {
        self.set_radius_top_right(radius);
        self
    }
    fn radius_bottom_right(mut self, radius: f32) -> Self {
        self.set_radius_bottom_right(radius);
        self
    }
    fn radius_bottom_left(mut self, radius: f32) -> Self {
        self.set_radius_bottom_left(radius);
        self
    }
    fn background<B: Into<Texture>>(mut self, texture: B) -> Self {
        self.set_background(texture);
        self
    }
    fn border_size(mut self, size: f32) -> Self {
        self.set_border_size(size);
        self
    }
    fn border_texture<T: Into<Texture>>(mut self, texture: T) -> Self {
        self.set_border_texture(texture);
        self
    }
    fn border<T: Into<Texture>>(mut self, texture: T, size: f32) -> Self {
        self.set_border(texture, size);
        self
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ShapeStyle {
    Background(Texture),
    Border(Texture, f32),
}

impl From<u32> for ShapeStyle {
    fn from(color: u32) -> Self {
        ShapeStyle::Background(color.into())
    }
}
