use crate::widgets::shapes::*;
use crate::*;
use scene::PrimitiveRef;
use std::f32::consts::FRAC_1_SQRT_2;
use tiny_skia::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Rectangle {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) texture: Texture,
    pub(crate) radius: (f32, f32, f32, f32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct BorderedRectangle {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) border_width: f32,
    pub(crate) texture: Texture,
    pub(crate) radius: (f32, f32, f32, f32),
}

impl Rectangle {
    pub fn square(size: f32) -> Self {
        Rectangle {
            width: size,
            height: size,
            radius: (0., 0., 0., 0.),
            texture: Texture::Transparent,
        }
    }
    pub fn new(width: f32, height: f32) -> Self {
        Rectangle {
            width,
            height,
            radius: (0., 0., 0., 0.),
            texture: Texture::Transparent,
        }
    }
    fn path(
        mut pb: PathBuilder,
        width: f32,
        height: f32,
        radius: (f32, f32, f32, f32),
    ) -> Option<Path> {
        let (x, y) = (0., 0.);
        let mut cursor = Coords::new(x, y);

        let (tl, tr, br, bl) = radius;

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
    fn minimum_height(&self) -> f32 {
        self.radius.0.max(self.radius.2)
    }
    fn minimum_width(&self) -> f32 {
        self.radius.1.max(self.radius.3)
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

impl GeometryExt for Rectangle {
    fn set_width(&mut self, width: f32) {
        self.width = width.round().max(self.minimum_width());
    }
    fn set_height(&mut self, height: f32) {
        self.height = height.round().max(self.minimum_height());
    }
}

impl Drawable for Rectangle {
    fn draw(&self, ctx: &mut DrawContext, transform: tiny_skia::Transform) {
        let pb = ctx.path_builder();
        if let Some(path) = Rectangle::path(pb, self.width, self.height, self.radius) {
            let (backend, clipmask) = ctx.draw_kit();
            if let Backend::Pixmap(dt) = backend {
                match &self.texture {
                    Texture::Color(color) => {
                        dt.fill_path(
                            &path,
                            &Paint {
                                shader: Shader::SolidColor(*color),
                                blend_mode: BlendMode::SourceOver,
                                anti_alias: true,
                                force_hq_pipeline: false,
                            },
                            FillRule::Winding,
                            transform,
                            clipmask,
                        );
                    }
                    Texture::Image(image) => {
                        let sx = self.width() / image.width();
                        let sy = self.height() / image.height();
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
                            FillRule::Winding,
                            transform,
                            clipmask,
                        );
                    }
                    Texture::LinearGradient(gradient) => {
                        let region = Region::new(0., 0., self.width(), self.height());
                        let start = match gradient.orientation {
                            Orientation::Horizontal => region.start(),
                            Orientation::Vertical => region.top_anchor(),
                        };
                        let end = match gradient.orientation {
                            Orientation::Horizontal => region.end(),
                            Orientation::Vertical => region.bottom_anchor(),
                        };
                        dt.fill_path(
                            &path,
                            &Paint {
                                shader: LinearGradient::new(
                                    start.into(),
                                    end.into(),
                                    gradient.stops.clone(),
                                    gradient.mode,
                                    Transform::from_scale(transform.sx, transform.sy),
                                )
                                .expect("Failed to build LinearGradient shader"),
                                blend_mode: BlendMode::SourceOver,
                                anti_alias: true,
                                force_hq_pipeline: false,
                            },
                            FillRule::Winding,
                            transform,
                            clipmask,
                        );
                    }
                    _ => {}
                }
            }
            ctx.reset(path.clear());
        } else {
            ctx.reset(PathBuilder::new());
        }
    }
}

impl Style for Rectangle {
    fn set_top_left_radius(&mut self, radius: f32) {
        self.radius.0 = radius;
    }
    fn set_top_right_radius(&mut self, radius: f32) {
        self.radius.1 = radius;
    }
    fn set_bottom_right_radius(&mut self, radius: f32) {
        self.radius.2 = radius;
    }
    fn set_bottom_left_radius(&mut self, radius: f32) {
        self.radius.3 = radius;
    }
    fn set_texture<B: Into<Texture>>(&mut self, background: B) {
        self.texture = background.into();
    }
}

impl<'p> From<&'p Rectangle> for PrimitiveRef<'p> {
    fn from(this: &'p Rectangle) -> Self {
        PrimitiveRef::Rectangle(this)
    }
}

impl<T> Widget<T> for Rectangle {
    fn draw_scene(&mut self, mut scene: Scene) {
        scene.insert_primitive(self)
    }
    fn sync<'d>(&'d mut self, _: &mut SyncContext<T>, _: Event<'d>) -> Damage {
        Damage::None
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        self.width = self
            .width
            .clamp(constraints.minimum_width(), constraints.maximum_width())
            .round();
        self.height = self
            .width
            .clamp(constraints.minimum_height(), constraints.maximum_height())
            .round();
        (self.width, self.height).into()
    }
}

impl Style for BorderedRectangle {
    fn set_top_left_radius(&mut self, radius: f32) {
        self.radius.0 = radius;
    }
    fn set_top_right_radius(&mut self, radius: f32) {
        self.radius.1 = radius;
    }
    fn set_bottom_right_radius(&mut self, radius: f32) {
        self.radius.2 = radius;
    }
    fn set_bottom_left_radius(&mut self, radius: f32) {
        self.radius.3 = radius;
    }
    fn set_texture<B: Into<Texture>>(&mut self, texture: B) {
        self.texture = texture.into();
    }
}

pub fn minimum_radius(radius: f32, border_width: f32) -> f32 {
    if border_width > radius {
        return 0.;
    }
    radius - (border_width / 2.)
}

impl Geometry for BorderedRectangle {
    fn width(&self) -> f32 {
        self.width + 2. * self.border_width
    }
    fn height(&self) -> f32 {
        self.height + 2. * self.border_width
    }
}

impl GeometryExt for BorderedRectangle {
    fn set_width(&mut self, width: f32) {
        self.width = width.round().max(self.minimum_width());
    }
    fn set_height(&mut self, height: f32) {
        self.height = height.round().max(self.minimum_height());
    }
}

impl BorderedRectangle {
    pub fn new(width: f32, height: f32) -> Self {
        BorderedRectangle {
            width,
            height,
            radius: (0., 0., 0., 0.),
            border_width: 0.,
            texture: Texture::Transparent,
        }
    }
    pub fn set_border_width(&mut self, border_width: f32) {
        self.border_width = border_width;
    }
    pub fn border_width(mut self, border_width: f32) -> Self {
        self.set_border_width(border_width);
        self
    }
    fn minimum_height(&self) -> f32 {
        self.radius.0.max(self.radius.2)
    }
    fn minimum_width(&self) -> f32 {
        self.radius.1.max(self.radius.3)
    }
}

impl<'p> From<&'p BorderedRectangle> for PrimitiveRef<'p> {
    fn from(this: &'p BorderedRectangle) -> Self {
        PrimitiveRef::BorderedRectangle(this)
    }
}

impl Drawable for BorderedRectangle {
    fn draw(&self, ctx: &mut DrawContext, transform: tiny_skia::Transform) {
        let pb = ctx.path_builder();
        if let Some(path) = Rectangle::path(
            pb,
            self.width + self.border_width,
            self.height + self.border_width,
            self.radius,
        ) {
            let (backend, clipmask) = ctx.draw_kit();
            if let Backend::Pixmap(dt) = backend {
                let stroke = Stroke {
                    width: self.border_width,
                    line_cap: LineCap::Butt,
                    line_join: LineJoin::Miter,
                    miter_limit: 4.,
                    dash: None,
                };
                match &self.texture {
                    Texture::Color(color) => {
                        dt.stroke_path(
                            &path,
                            &Paint {
                                shader: Shader::SolidColor(*color),
                                blend_mode: BlendMode::SourceOver,
                                anti_alias: true,
                                force_hq_pipeline: false,
                            },
                            &stroke,
                            transform.pre_translate(self.border_width / 2., self.border_width / 2.),
                            clipmask,
                        );
                    }
                    Texture::Image(image) => {
                        let sx = self.width() / image.width();
                        let sy = self.height() / image.height();
                        dt.stroke_path(
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
                            &stroke,
                            transform.pre_translate(self.border_width / 2., self.border_width / 2.),
                            clipmask,
                        );
                    }
                    Texture::LinearGradient(gradient) => {
                        let region = Region::new(0., 0., self.width(), self.height());
                        let start = match gradient.orientation {
                            Orientation::Horizontal => region.start(),
                            Orientation::Vertical => region.top_anchor(),
                        };
                        let end = match gradient.orientation {
                            Orientation::Horizontal => region.end(),
                            Orientation::Vertical => region.bottom_anchor(),
                        };
                        dt.stroke_path(
                            &path,
                            &Paint {
                                shader: LinearGradient::new(
                                    start.into(),
                                    end.into(),
                                    gradient.stops.clone(),
                                    gradient.mode,
                                    Transform::from_scale(transform.sx, transform.sy),
                                )
                                .expect("Failed to build LinearGradient shader"),
                                blend_mode: BlendMode::SourceOver,
                                anti_alias: true,
                                force_hq_pipeline: false,
                            },
                            &stroke,
                            transform.pre_translate(self.border_width / 2., self.border_width / 2.),
                            clipmask,
                        );
                    }
                    _ => {}
                }
            }
            ctx.reset(path.clear());
        } else {
            ctx.reset(PathBuilder::new());
        }
    }
}
