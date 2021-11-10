use crate::widgets::primitives::*;
use crate::*;
use widgets::u32_to_source;

impl Style {
    pub fn fill(color: u32) -> Self {
        Style::Fill(u32_to_source(color))
    }
    pub fn border(color: u32, size: f32) -> Self {
        Style::Border(u32_to_source(color), size)
    }
    pub fn is_empty(&self) -> bool {
        if let Style::Empty = self {
            true
        } else {
            false
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rectangle {
    pub width: f32,
    pub height: f32,
    pub style: Style,
    pub radius: [f32; 4],
}

impl Rectangle {
    pub fn new(width: f32, height: f32, style: Style) -> Self {
        Rectangle {
            width,
            height,
            style,
            radius: [0.; 4],
        }
    }
    pub fn square(size: f32, style: Style) -> Self {
        Rectangle {
            width: size,
            height: size,
            style,
            radius: [0.; 4],
        }
    }
    pub fn empty(width: f32, height: f32) -> Self {
        Rectangle {
            width,
            height,
            style: Style::Empty,
            radius: [0.; 4],
        }
    }
    pub fn set_radius(&mut self, radius: [f32; 4]) {
        self.radius = radius;
    }
}

impl Geometry for Rectangle {
    fn width(&self) -> f32 {
        self.width
    }
    fn height(&self) -> f32 {
        self.height
    }
}

impl Drawable for Rectangle {
    fn set_color(&mut self, color: u32) {
        if let Style::Border(source, _) = &mut self.style {
            *source = u32_to_source(color);
        } else if let Style::Fill(source) = &mut self.style {
            *source = u32_to_source(color);
        }
    }
    fn draw(&self, ctx: &mut Context, x: f32, y: f32) {
        if !self.style.is_empty() {
            ctx.draw_rectangle(x, y, self.width(), self.height(), self.radius, &self.style);
        }
    }
}

impl Widget for Rectangle {
    fn create_node(&self, x: f32, y: f32) -> RenderNode {
        RenderNode::Widget(Damage::from_rectangle(x, y, *self))
    }
    fn roundtrip<'d>(&'d mut self, _wx: f32, _wy: f32, ctx: &mut Context, _dispatch: &Dispatch) {
    }
}

