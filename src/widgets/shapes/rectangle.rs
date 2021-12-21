use crate::widgets::shapes::*;
use crate::*;
use scene::RenderNode;
use std::f32::consts::FRAC_1_SQRT_2;
use std::ops::DerefMut;
use tiny_skia::*;
use widgets::u32_to_source;

impl ShapeStyle {
    pub fn solid(color: u32) -> Self {
        ShapeStyle::Background(Background::Color(u32_to_source(color)))
    }
    pub fn border(color: u32, size: f32) -> Self {
        ShapeStyle::Border(u32_to_source(color), size)
    }
    pub fn background(&self) -> Background {
        match self {
            ShapeStyle::Background(background) => background.clone(),
            _ => Background::Transparent,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rectangle {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) style: ShapeStyle,
    pub(crate) radius: (f32, f32, f32, f32),
}

impl From<Region> for Rectangle {
    fn from(region: Region) -> Self {
        Rectangle {
            width: region.x,
            height: region.y,
            style: ShapeStyle::Background(Background::Transparent),
            radius: (0., 0., 0., 0.),
        }
    }
}

impl Rectangle {
    pub fn square(size: f32, style: ShapeStyle) -> Self {
        Rectangle {
            width: size,
            height: size,
            style,
            radius: (0., 0., 0., 0.),
        }
    }
    pub fn new(width: f32, height: f32, style: ShapeStyle) -> Self {
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
            style: ShapeStyle::Background(Background::Transparent),
        }
    }
    pub fn get_style(&self) -> &ShapeStyle {
        &self.style
    }
    pub fn get_radius(&self) -> (f32, f32, f32, f32) {
        self.radius
    }
    pub fn is_opaque(&self) -> bool {
        match &self.style {
            ShapeStyle::Background(background) => match background {
                Background::Transparent => false,
                Background::Color(source) => source.is_opaque(),
                _ => false,
            },
            ShapeStyle::Border(_, _) => false,
        }
    }
    pub fn path(&self) -> Option<Path> {
        let mut width = self.width;
        let mut height = self.height;
        let (mut x, mut y) = (0., 0.);
        if let ShapeStyle::Border(_, size) = &self.style {
            x += (size / 2.).ceil();
            y += (size / 2.).ceil();
            width += size;
            height += size;
        }
        let mut cursor = Coords::new(x, y);
        let mut pb = PathBuilder::new();

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

        pb.finish()
    }
}

impl Geometry for Rectangle {
    fn width(&self) -> f32 {
        self.width
            + if let ShapeStyle::Border(_, size) = &self.style {
                2. * *size
            } else {
                0.
            }
    }
    fn height(&self) -> f32 {
        self.height
            + if let ShapeStyle::Border(_, size) = &self.style {
                2. * *size
            } else {
                0.
            }
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        if width.is_sign_negative() {
            return Err(self.width());
        }
        self.width = self
            .radius
            .0
            .max(width.round())
            .max(self.radius.1)
            .max(self.radius.2)
            .max(self.radius.3);
        if let ShapeStyle::Background(background) = &mut self.style {
            if let Background::Image(_, img) = background {
                img.set_width(self.width)?;
            }
        }
        return Ok(());
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        if height.is_sign_negative() {
            return Err(self.height());
        }
        self.height = self
            .radius
            .0
            .max(height.round())
            .max(self.radius.1)
            .max(self.radius.2)
            .max(self.radius.3);
        if let ShapeStyle::Background(background) = &mut self.style {
            if let Background::Image(_, img) = background {
                img.set_height(self.height)?;
            }
        }
        return Ok(());
    }
}

impl Primitive for Rectangle {
    fn apply_background(&self, background: scene::Background) -> scene::PrimitiveType {
        let mut rect = self.clone();
        rect.style = ShapeStyle::Background(background);
        rect.into()
    }
    fn get_background(&self) -> scene::Background {
        self.style.background()
    }
    fn into_primitive(&self) -> scene::PrimitiveType {
        self.clone().into()
    }
    fn contains(&self, region: &scene::Region) -> bool {
        let (tl, tr, br, bl) = self.radius;
        let max = tl.max(tr).max(br).max(bl);
        let radius = (max - (max * FRAC_1_SQRT_2)).floor();
        region.x >= radius
            && region.y >= radius
            && region.width <= self.width - radius
            && region.height <= self.height - radius
    }
    fn draw_with_transform_clip(
        &self,
        ctx: &mut DrawContext,
        transform: tiny_skia::Transform,
        clip: Option<&tiny_skia::ClipMask>,
    ) {
        if let Some(path) = self.path() {
            let width = self.width;
            let (x, y) = (0., 0.);
            if let Backend::Pixmap(dt) = ctx.deref_mut() {
                match &self.style {
                    ShapeStyle::Background(background) => match background {
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
                                transform,
                                clip,
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
                                transform,
                                clip,
                            );
                        }
                        Background::LinearGradient {
                            start: _,
                            end: _,
                            angle,
                            stops,
                            mode,
                        } => {
                            if let Some(grad) = LinearGradient::new(
                                Point::from_xy(x, y),
                                Point::from_xy(x + width, y + self.height * angle.tan()),
                                stops.as_ref().to_vec(),
                                *mode,
                                Transform::identity(),
                            ) {
                                dt.fill_path(
                                    &path,
                                    &Paint {
                                        shader: grad,
                                        blend_mode: BlendMode::SourceOver,
                                        anti_alias: true,
                                        force_hq_pipeline: false,
                                    },
                                    FillRule::EvenOdd,
                                    transform,
                                    clip,
                                );
                            }
                        }
                        _ => {}
                    },
                    ShapeStyle::Border(color, border) => {
                        let stroke = Stroke {
                            width: *border,
                            line_cap: LineCap::Square,
                            line_join: LineJoin::Miter,
                            miter_limit: 4.,
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
                            transform,
                            clip,
                        );
                    }
                }
            }
        }
    }
}

impl Style for Rectangle {
    fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.radius = (tl, tr, br, bl);
    }
    fn radius(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        self.radius = (tl, tr, br, bl);
        self
    }
    fn set_background<B: Into<Background>>(&mut self, background: B) {
        let mut background = background.into();
        if let Background::Image(_, img) = &mut background {
            img.set_size(self.width(), self.height()).unwrap();
        }
        match &mut background {
            Background::Image(_, image) => {
                image.set_size(self.width(), self.height()).unwrap();
            }
            _ => {}
        }
        self.style = ShapeStyle::Background(background);
    }
    fn background<B: Into<Background>>(mut self, background: B) -> Self {
        self.set_background(background);
        self
    }
    fn set_border(&mut self, color: u32, width: f32) {
        self.style = ShapeStyle::border(color, width);
    }
    fn border(mut self, color: u32, width: f32) -> Self {
        self.style = ShapeStyle::border(color, width);
        self
    }
    fn set_border_color(&mut self, color: u32) {
        if let ShapeStyle::Border(_, width) = self.style {
            self.style = ShapeStyle::border(color, width);
        } else {
            self.style = ShapeStyle::border(color, 0.);
        }
    }
    fn border_color(mut self, color: u32) -> Self {
        if let ShapeStyle::Border(_, width) = self.style {
            self.style = ShapeStyle::border(color, width);
        } else {
            self.style = ShapeStyle::border(color, 0.);
        }
        self
    }
    fn set_border_width(&mut self, width: f32) {
        if let ShapeStyle::Border(color, _) = self.style {
            self.style = ShapeStyle::Border(color, width);
        } else {
            self.style = ShapeStyle::border(0, width);
        }
    }
    fn border_width(mut self, width: f32) -> Self {
        if let ShapeStyle::Border(color, _) = self.style {
            self.style = ShapeStyle::Border(color, width);
        } else {
            self.style = ShapeStyle::border(0, width);
        }
        self
    }
}

impl Widget for Rectangle {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        if let ShapeStyle::Background(background) = &mut self.style {
            match background {
                Background::Image(coords, _) => {
                    coords.x = x;
                    coords.y = y;
                }
                Background::LinearGradient {
                    start,
                    end,
                    angle,
                    stops: _,
                    mode: _,
                } => {
                    start.x = x;
                    start.y = y;
                    end.x = x + self.width;
                    end.y = y + self.height * angle.tan();
                }
                _ => {}
            }
        }
        RenderNode::Instruction(Instruction::new(x, y, self.clone()))
    }
    fn sync<'d>(&'d mut self, _ctx: &mut SyncContext, _event: Event) -> Damage {
        Damage::None
    }
}
