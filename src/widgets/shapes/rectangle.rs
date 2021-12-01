use crate::widgets::shapes::*;
use crate::*;
use scene::RenderNode;
use std::f32::consts::FRAC_1_SQRT_2;
use std::ops::DerefMut;
use tiny_skia::*;
use widgets::u32_to_source;

impl Style {
    pub fn solid(color: u32) -> Self {
        Style::Background(Background::Color(u32_to_source(color)))
    }
    pub fn border(color: u32, size: f32) -> Self {
        Style::Border(u32_to_source(color), size)
    }
    pub fn background(&self) -> Background {
        match self {
            Style::Background(background) => background.clone(),
            _ => Background::Transparent,
        }
    }
    pub fn source(&self) -> Color {
        match self {
            Style::Background(background) => match background {
                Background::Transparent => u32_to_source(0),
                Background::Color(source) => *source,
                _ => panic!("Background cannot return a color"),
            },
            Style::Border(source, _) => *source,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rectangle {
    width: f32,
    height: f32,
    style: Style,
    radius: (f32, f32, f32, f32),
}

impl Rectangle {
    pub fn square(size: f32, style: Style) -> Self {
        Rectangle {
            width: size,
            height: size,
            style,
            radius: (0., 0., 0., 0.),
        }
    }
    pub fn new(width: f32, height: f32, style: Style) -> Self {
        Rectangle {
            width,
            height,
            style,
            radius: (0., 0., 0., 0.),
        }
    }
    pub fn empty(width: f32, height: f32) -> Self {
        Rectangle {
            width,
            height,
            radius: (0., 0., 0., 0.),
            style: Style::Background(Background::Transparent),
        }
    }
    pub fn get_style(&self) -> &Style {
        &self.style
    }
    pub fn get_radius(&self) -> (f32, f32, f32, f32) {
        self.radius
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
        self.width = self
            .radius
            .0
            .max(width.round())
            .max(self.radius.1)
            .max(self.radius.2)
            .max(self.radius.3);
        if let Style::Background(background) = &mut self.style {
            if let Background::Image(_, img) = background {
                img.set_width(width)?;
            }
        }
        return Ok(());
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.height = self
            .radius
            .0
            .max(height.round())
            .max(self.radius.1)
            .max(self.radius.2)
            .max(self.radius.3);
        if let Style::Background(background) = &mut self.style {
            if let Background::Image(_, img) = background {
                img.set_height(height)?;
            }
        }
        return Ok(());
    }
}

impl Primitive for Rectangle {
    fn draw(&self, x: f32, y: f32, ctx: &mut DrawContext) {
        let width = self.width();
        let height = self.height();
        let mut pb = PathBuilder::new();
        let mut cursor = Coords::new(x, y);

        let (tl, tr, br, bl) = self.radius;

        // Positioning the cursor
        cursor.y += tl;
        pb.move_to(cursor.x, cursor.y);

        // Drawing the outline
        pb.cubic_to(
            cursor.x,
            cursor.y,
            cursor.x,
            cursor.y - FRAC_1_SQRT_2 * tl,
            {
                cursor.x += tl;
                cursor.x
            },
            {
                cursor.y -= tl;
                cursor.y
            },
        );
        pb.line_to(
            {
                cursor.x = x + width - tr;
                cursor.x
            },
            cursor.y,
        );
        pb.cubic_to(
            cursor.x,
            cursor.y,
            cursor.x + FRAC_1_SQRT_2 * tr,
            cursor.y,
            {
                cursor.x += tr;
                cursor.x
            },
            {
                cursor.y += tr;
                cursor.y
            },
        );
        pb.line_to(cursor.x, {
            cursor.y = y + height - br;
            cursor.y
        });
        pb.cubic_to(
            cursor.x,
            cursor.y,
            cursor.x,
            cursor.y + FRAC_1_SQRT_2 * br,
            {
                cursor.x -= br;
                cursor.x
            },
            {
                cursor.y += br;
                cursor.y
            },
        );
        pb.line_to(
            {
                cursor.x = x + bl;
                cursor.x
            },
            cursor.y,
        );
        pb.cubic_to(
            cursor.x,
            cursor.y,
            cursor.x - FRAC_1_SQRT_2 * bl,
            cursor.y,
            {
                cursor.x -= bl;
                cursor.x
            },
            {
                cursor.y -= bl;
                cursor.y
            },
        );

        // Closing path
        pb.close();

        if let Some(path) = pb.finish() {
            if let Backend::Pixmap(dt) = ctx.deref_mut() {
                match &self.style {
                    Style::Background(background) => match background {
                        Background::Color(color) => {
                            dt.fill_path(
                                &path,
                                &Paint {
                                    shader: Shader::SolidColor(color.clone()),
                                    blend_mode: BlendMode::SourceOver,
                                    anti_alias: true,
                                    force_hq_pipeline: false,
                                },
                                FillRule::EvenOdd,
                                Transform::identity(),
                                None,
                            );
                        }
                        Background::Image(_, image) => {
                            let (sx, sy) = image.scale();
                            dt.fill_path(
                                &path,
                                &Paint {
                                    shader: Pattern::new(
                                        image.pixmap(),
                                        SpreadMode::Repeat,
                                        FilterQuality::Bilinear,
                                        1.0,
                                        Transform::from_scale(sx, sy),
                                    ),
                                    blend_mode: BlendMode::SourceOver,
                                    anti_alias: true,
                                    force_hq_pipeline: false,
                                },
                                FillRule::EvenOdd,
                                Transform::identity(),
                                None,
                            );
                        }
                        _ => {}
                    },
                    Style::Border(color, border) => {
                        let stroke = Stroke {
                            width: *border,
                            line_cap: LineCap::Butt,
                            line_join: LineJoin::Miter,
                            miter_limit: 10.,
                            dash: None,
                        };
                        dt.stroke_path(
                            &path,
                            &Paint {
                                shader: Shader::SolidColor(*color),
                                blend_mode: BlendMode::SourceOver,
                                anti_alias: true,
                                force_hq_pipeline: false,
                            },
                            &stroke,
                            Transform::identity(),
                            None,
                        )
                        .unwrap();
                    }
                }
            }
        }
    }
}

impl Shape for Rectangle {
    fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.radius = (tl, tr, br, bl);
    }
    fn radius(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        self.radius = (tl, tr, br, bl);
        self
    }
    fn set_background<B: Into<Background>>(&mut self, background: B) {
        self.style = Style::Background(background.into());
    }
    fn background<B: Into<Background>>(mut self, background: B) -> Self {
        let mut background = background.into();
        if let Background::Image(_, img) = &mut background {
            img.set_size(self.width(), self.height()).unwrap();
        }
        self.style = Style::Background(background);
        self
    }
    fn set_border(&mut self, color: u32, width: f32) {
        self.style = Style::border(color, width);
    }
    fn border(mut self, color: u32, width: f32) -> Self {
        self.style = Style::border(color, width);
        self
    }
    fn set_border_color(&mut self, color: u32) {
        if let Style::Border(_, width) = self.style {
            self.style = Style::border(color, width);
        } else {
            self.style = Style::border(color, 0.);
        }
    }
    fn border_color(mut self, color: u32) -> Self {
        if let Style::Border(_, width) = self.style {
            self.style = Style::border(color, width);
        } else {
            self.style = Style::border(color, 0.);
        }
        self
    }
    fn set_border_width(&mut self, width: f32) {
        if let Style::Border(color, _) = self.style {
            self.style = Style::Border(color, width);
        } else {
            self.style = Style::border(0, width);
        }
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
        if let Style::Background(backgorund) = &mut self.style {
            if let Background::Image(coords, _) = backgorund {
                coords.x = x;
                coords.y = y;
            }
        }
        RenderNode::Instruction(Instruction::new(x, y, self.clone()))
    }
    fn sync<'d>(&'d mut self, _ctx: &mut SyncContext, _event: Event) {}
}
