use crate::widgets::shapes::*;
use crate::*;
use scene::{Background, RenderNode};
use std::f32::consts::PI;
use widgets::u32_to_source;

impl Style {
    pub fn fill(color: u32) -> Self {
        Style::Fill(u32_to_source(color))
    }
    pub fn border(color: u32, size: f32) -> Self {
        Style::Border(u32_to_source(color), size)
    }
    pub fn source(&self) -> SolidSource {
        match self {
            Style::Border(source, _) => *source,
            Style::Fill(source) => *source,
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
    pub fn square(size: f32, style: Style) -> Self {
        Rectangle {
            width: size,
            height: size,
            style,
            radius: [0.; 4],
        }
    }
    pub fn new(width: f32, height: f32, style: Style) -> Self {
        Rectangle {
            width,
            height,
            style,
            radius: [0.; 4],
        }
    }
    pub fn empty() -> Self {
        Rectangle {
            width: 0.,
            height: 0.,
            style: Style::fill(0),
            radius: [0.; 4],
        }
    }
}

impl Geometry for Rectangle {
    fn width(&self) -> f32 {
        self.width
    }
    fn height(&self) -> f32 {
        self.height
    }
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        if width > 0. && height > 0. {
            self.width = width;
            self.height = height;
        } else {
            return Err((self.width, self.height));
        }
        Ok(())
    }
}

impl Primitive for Rectangle {
    fn draw(&self, x: f32, y: f32, ctx: &mut Context) {
        let mut width = self.width();
        let mut height = self.height();
        let mut pb = PathBuilder::new();
        let mut cursor = match self.style {
            Style::Border(_, border) => {
                width -= border;
                height -= border;
                (x + border / 2., y + border / 2.)
            }
            _ => (x, y),
        };

        // Sides length
        let top = width - self.radius[0] - self.radius[1];
        let right = height - self.radius[1] - self.radius[2];
        let left = height - self.radius[0] - self.radius[3];
        let bottom = width - self.radius[2] - self.radius[3];

        // Positioning the cursor
        cursor.0 += self.radius[0];
        cursor.1 += self.radius[0];

        // Drawing the outline
        pb.arc(cursor.0, cursor.1, self.radius[0], PI, PI / 2.);
        cursor.0 += top;
        cursor.1 -= self.radius[0];
        pb.line_to(cursor.0, cursor.1);
        cursor.1 += self.radius[1];
        pb.arc(cursor.0, cursor.1, self.radius[1], -PI / 2., PI / 2.);
        cursor.0 += self.radius[1];
        cursor.1 += right;
        pb.line_to(cursor.0, cursor.1);
        cursor.0 -= self.radius[2];
        pb.arc(cursor.0, cursor.1, self.radius[2], 0., PI / 2.);
        cursor.1 += self.radius[2];
        cursor.0 -= bottom;
        pb.line_to(cursor.0, cursor.1);
        cursor.1 -= self.radius[3];
        pb.arc(cursor.0, cursor.1, self.radius[3], PI / 2., PI / 2.);
        cursor.0 -= self.radius[3];
        cursor.1 -= left;
        pb.line_to(cursor.0, cursor.1);

        // Closing path
        pb.close();
        let path = pb.finish();

        ctx.draw_path(path, &self.style);
    }
}

impl Shape for Rectangle {
    fn set_radius(mut self, radius: f32) -> Self {
        self.radius = [radius; 4];
        self
    }
    fn set_background(mut self, color: u32) -> Self {
        self.style = Style::fill(color);
        self
    }
    fn set_border(mut self, color: u32, width: f32) -> Self {
        self.style = Style::border(color, width);
        self
    }
    fn set_border_color(mut self, color: u32) -> Self {
        if let Style::Border(_, width) = self.style {
            self.style = Style::border(color, width);
        } else {
            self.style = Style::border(color, 0.);
        }
        self
    }
    fn set_border_width(mut self, width: f32) -> Self {
        if let Style::Border(color, _) = self.style {
            self.style = Style::Border(color, width);
        } else {
            self.style = Style::border(0, width);
        }
        self
    }
}

impl Widget for Rectangle {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        RenderNode::Instruction(Instruction::new(x, y, *self))
    }
    fn sync<'d>(&'d mut self, ctx: &mut Context, event: Event) {}
}
