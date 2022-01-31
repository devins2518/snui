use crate::widgets::shapes::*;
use crate::*;
use scene::RenderNode;
use std::f32::consts::FRAC_1_SQRT_2;
use std::ops::DerefMut;
use tiny_skia::*;
use widgets::u32_to_source;

impl ShapeStyle {
    pub fn solid(color: u32) -> Self {
        ShapeStyle::Background(Texture::Color(u32_to_source(color)))
    }
    pub fn border(color: u32, size: f32) -> Self {
        ShapeStyle::Border(u32_to_source(color).into(), size)
    }
    pub fn background(&self) -> Texture {
        match self {
            ShapeStyle::Background(background) => background.clone(),
            _ => Texture::Transparent,
        }
    }
    pub fn texture(&self) -> Texture {
        match self {
            ShapeStyle::Background(background) => background.clone(),
            ShapeStyle::Border(texture, _) => texture.clone(),
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
            style: ShapeStyle::Background(Texture::Transparent),
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
            style: ShapeStyle::Background(Texture::Transparent),
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
                Texture::Transparent => false,
                Texture::Color(source) => source.is_opaque(),
                _ => false,
            },
            ShapeStyle::Border(_, _) => false,
        }
    }
    pub fn path(&self, mut pb: PathBuilder) -> Option<Path> {
        let mut width = self.width;
        let mut height = self.height;
        let (mut x, mut y) = (0., 0.);
        if let ShapeStyle::Border(_, size) = &self.style {
            x += size / 2.;
            y += size / 2.;
            width += size;
            height += size;
        }
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
        return Ok(());
    }
}

impl Primitive for Rectangle {
    fn apply_texture(&self, texture: scene::Texture) -> scene::PrimitiveType {
        let mut rect = self.clone();
        match &mut rect.style {
            ShapeStyle::Border(border, _) => *border = texture,
            ShapeStyle::Background(background) => *background = texture,
        }
        rect.into()
    }
    fn get_texture(&self) -> scene::Texture {
        self.style.texture()
    }
    fn primitive_type(&self) -> scene::PrimitiveType {
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
        let pb = ctx.path_builder();
        if let Some(path) = self.path(pb) {
            let width = self.width;
            let (x, y) = (0., 0.);
            if let Backend::Pixmap(dt) = ctx.deref_mut() {
                match &self.style {
                    ShapeStyle::Background(background) => match background {
                        Texture::Color(color) => {
                            dt.fill_path(
                                &path,
                                &Paint {
                                    shader: Shader::SolidColor(color.clone()),
                                    blend_mode: BlendMode::SourceOver,
                                    anti_alias: true,
                                    force_hq_pipeline: false,
                                },
                                FillRule::Winding,
                                transform,
                                clip,
                            );
                        }
                        Texture::Image(_, image) => {
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
                                FillRule::Winding,
                                transform,
                                clip,
                            );
                        }
                        Texture::LinearGradient {
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
                                    FillRule::Winding,
                                    transform,
                                    clip,
                                );
                            }
                        }
                        _ => {}
                    },
                    ShapeStyle::Border(texture, border) => {
                        let stroke = Stroke {
                            width: *border,
                            line_cap: LineCap::Butt,
                            line_join: LineJoin::Miter,
                            miter_limit: 4.,
                            dash: None,
                        };
                        dt.stroke_path(
                            &path,
                            &Paint {
                                shader: match texture {
                                    Texture::Color(color) => Shader::SolidColor(*color),
                                    Texture::LinearGradient {
                                        angle, mode, stops, ..
                                    } => LinearGradient::new(
                                        Point::from_xy(x, y),
                                        Point::from_xy(x + width, y + self.height * angle.tan()),
                                        stops.as_ref().to_vec(),
                                        *mode,
                                        Transform::identity(),
                                    )
                                    .unwrap(),
                                    Texture::Image(_, image) => {
                                        let (sx, sy) = image.scale();
                                        Pattern::new(
                                            image.pixmap(),
                                            SpreadMode::Repeat,
                                            FilterQuality::Bilinear,
                                            1.0,
                                            Transform::from_scale(sx, sy),
                                        )
                                    }
                                    &Texture::Composite(_) => {
                                        panic!("Composite texture cannot be used to draw borders.")
                                    }
                                    _ => Shader::SolidColor(Color::TRANSPARENT),
                                },
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
            ctx.reset(path.clear());
        } else {
            ctx.reset(PathBuilder::new());
        }
    }
}

impl Style for Rectangle {
    fn set_radius_top_left(&mut self, radius: f32) {
        self.radius.0 = radius;
    }
    fn set_radius_top_right(&mut self, radius: f32) {
        self.radius.1 = radius;
    }
    fn set_radius_bottom_right(&mut self, radius: f32) {
        self.radius.2 = radius;
    }
    fn set_radius_bottom_left(&mut self, radius: f32) {
        self.radius.3 = radius;
    }
    fn set_background<B: Into<Texture>>(&mut self, background: B) {
        let mut background = background.into();
        if let Texture::Image(_, img) = &mut background {
            img.set_size(self.width(), self.height()).unwrap();
        }
        match &mut background {
            Texture::Image(_, image) => {
                image.set_size(self.width(), self.height()).unwrap();
            }
            _ => {}
        }
        self.style = ShapeStyle::Background(background);
    }
    fn set_border_texture<T: Into<Texture>>(&mut self, texture: T) {
        if let ShapeStyle::Border(_, width) = self.style {
            self.style = ShapeStyle::Border(texture.into(), width);
        } else {
            self.style = ShapeStyle::Border(texture.into(), 0.);
        }
    }
    fn set_border_size(&mut self, size: f32) {
        if let ShapeStyle::Border(_, border_size) = &mut self.style {
            *border_size = size;
        } else {
            self.style = ShapeStyle::Border(Texture::Transparent, size);
        }
    }
}

impl<D> Widget<D> for Rectangle {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        if transform.is_scale_translate() {
            if let ShapeStyle::Background(background) = &mut self.style {
                match background {
                    Texture::Image(coords, image) => {
                        coords.x = transform.tx;
                        coords.y = transform.ty;
                        image.set_size(self.width, self.height).unwrap();
                    }
                    Texture::LinearGradient {
                        start,
                        end,
                        angle,
                        stops: _,
                        mode: _,
                    } => {
                        start.x = transform.tx;
                        start.y = transform.ty;
                        end.x = transform.tx + self.width;
                        end.y = transform.ty + self.height * angle.tan();
                    }
                    _ => {}
                }
            }
            return Instruction::new(transform, self.clone()).into();
        }
        RenderNode::None
    }
    fn sync<'d>(&'d mut self, _: &mut SyncContext<D>, _: Event<'d>) -> Damage {
        Damage::None
    }
}
