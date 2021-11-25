use crate::widgets::shapes::*;
use crate::*;
use scene::{RenderNode};
use std::f32::consts::{FRAC_PI_2, PI};
use std::ops::{DerefMut};
use widgets::u32_to_source;

const DRAW_OPTIONS: DrawOptions = DrawOptions {
    blend_mode: BlendMode::SrcOver,
    alpha: 1.,
    antialias: AntialiasMode::Gray,
};

const ATOP_OPTIONS: DrawOptions = DrawOptions {
    alpha: 1.,
    blend_mode: BlendMode::SrcAtop,
    antialias: AntialiasMode::Gray,
};

impl Style {
    pub fn solid(color: u32) -> Self {
        Style::Solid(u32_to_source(color))
    }
    pub fn border(color: u32, size: f32) -> Self {
        Style::Border(u32_to_source(color), size)
    }
    pub fn source(&self) -> SolidSource {
        match self {
            Style::Solid(source) => *source,
            Style::Border(source, _) => *source,
            _ => panic!("Gradient doesn't own a SolidSource"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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
            style: Style::solid(0),
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
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        if width.is_sign_positive() {
            self.width = width.round();
            return Ok(());
        }
        Err(self.width)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        if height.is_sign_positive() {
            self.height = height.round();
            return Ok(());
        }
        Err(self.height)
    }
}

impl Primitive for Rectangle {
    fn draw(&self, x: f32, y: f32, ctx: &mut DrawContext) {
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

        let size = self.width.min(self.height) / 2.;
        let radius = [
            self.radius[0].min(size),
            self.radius[1].min(size),
            self.radius[2].min(size),
            self.radius[3].min(size),
        ];

        // Sides length
        let top = width - radius[0] - radius[1];
        let right = height - radius[1] - radius[2];
        let left = height - radius[0] - radius[3];
        let bottom = width - radius[2] - radius[3];

        // Positioning the cursor
        cursor.0 += radius[0];
        cursor.1 += radius[0];

        // Drawing the outline
        pb.arc(cursor.0, cursor.1, radius[0], PI, FRAC_PI_2);
        cursor.0 += top;
        cursor.1 -= radius[0];
        pb.line_to(cursor.0, cursor.1);
        cursor.1 += radius[1];
        pb.arc(cursor.0, cursor.1, radius[1], -FRAC_PI_2, FRAC_PI_2);
        cursor.0 += radius[1];
        cursor.1 += right;
        pb.line_to(cursor.0, cursor.1);
        cursor.0 -= radius[2];
        pb.arc(cursor.0, cursor.1, radius[2], 0., FRAC_PI_2);
        cursor.1 += radius[2];
        cursor.0 -= bottom;
        pb.line_to(cursor.0, cursor.1);
        cursor.1 -= radius[3];
        pb.arc(cursor.0, cursor.1, radius[3], FRAC_PI_2, FRAC_PI_2);
        cursor.0 -= radius[3];
        cursor.1 -= left;
        pb.line_to(cursor.0, cursor.1);

        // Closing path
        pb.close();
        let path = pb.finish();

        if let Backend::Raqote(dt) = ctx.deref_mut() {
            match &self.style {
                Style::Solid(source) => {
                    dt.fill(&path, &Source::Solid(*source), &DRAW_OPTIONS);
                }
                Style::Border(source, border) => {
                    let stroke = StrokeStyle {
                        width: *border,
                        cap: LineCap::Butt,
                        join: LineJoin::Miter,
                        miter_limit: 100.,
                        dash_array: Vec::new(),
                        dash_offset: 0.,
                    };
                    dt.stroke(&path, &Source::Solid(*source), &stroke, &ATOP_OPTIONS);
                }
                Style::LinearGradient(grad, spread) => {
                    let source = Source::new_linear_gradient(
                        grad.as_ref().clone(),
                        Point::new(x, y),
                        Point::new(x + self.width, y + self.height),
                        *spread,
                    );
                    dt.fill(&path, &source, &DRAW_OPTIONS);
                }
                Style::RadialGradient(grad, spread, rad) => {
                    let source = Source::new_radial_gradient(
                        grad.as_ref().clone(),
                        Point::new(x, y),
                        *rad,
                        *spread,
                    );
                    dt.fill(&path, &source, &DRAW_OPTIONS);
                }
            }
        }
    }
}

impl Shape for Rectangle {
    fn radius(mut self, radius: f32) -> Self {
        self.radius = [radius; 4];
        self
    }
    fn background(mut self, color: u32) -> Self {
        self.style = Style::solid(color);
        self
    }
    fn border(mut self, color: u32, width: f32) -> Self {
        self.style = Style::border(color, width);
        self
    }
    fn border_color(mut self, color: u32) -> Self {
        if let Style::Border(_, width) = self.style {
            self.style = Style::border(color, width);
        } else {
            self.style = Style::border(color, 0.);
        }
        self
    }
    fn border_width(mut self, width: f32) -> Self {
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
        RenderNode::Instruction(Instruction::new(x, y, self.clone()))
    }
    fn sync<'d>(&'d mut self, _ctx: &mut SyncContext, _event: Event) {}
}
