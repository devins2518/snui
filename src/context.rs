use crate::widgets::text::{Font, GlyphCache, GlyphPosition};
use crate::*;
use euclid::default::{Box2D, Point2D};
use lyon_geom::euclid::{point2, vec2, Angle};
use raqote::*;
use std::any::Any;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::ops::{Deref, DerefMut};
use widgets::primitives::Style;

#[derive(Debug, Clone, Copy,PartialEq, Eq)]
pub enum DamageType {
    Full,
    Resize,
    Partial,
}

pub enum Backend {
    Raqote(DrawTarget),
}

const ATOP_OPTIONS: DrawOptions = DrawOptions {
    blend_mode: BlendMode::SrcAtop,
    alpha: 1.,
    antialias: AntialiasMode::Gray,
};

const DRAW_OPTIONS: DrawOptions = DrawOptions {
    blend_mode: BlendMode::SrcOver,
    alpha: 1.,
    antialias: AntialiasMode::Gray,
};

#[derive(Debug, Copy, Clone)]
pub struct Region {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

fn add_region(vec: &mut Vec<Region>, x: f32, y: f32, width: f32, height: f32) {
    if let Some(last) = vec.last() {
        if !(last.x <= x
            && last.y <= y
            && last.x + last.width >= x + width
            && last.y + last.height >= y + height)
        {
            vec.push(Region {
                x,
                y,
                width,
                height,
            });
        }
    } else {
        vec.push(Region {
            x,
            y,
            width,
            height,
        });
    }
}

pub struct Context {
    pub running: bool,
    backend: Backend,
    input_regions: Vec<Region>,
    damage_type: DamageType,
    damage_regions: Vec<Region>,
    values: HashMap<String, Box<dyn Any>>,
    font_cache: HashMap<String, GlyphCache>,
}

impl Context {
    pub fn new(backend: Backend) -> Self {
        Self {
            backend,
            running: true,
            values: HashMap::new(),
            input_regions: Vec::new(),
            damage_regions: Vec::new(),
            font_cache: HashMap::new(),
            damage_type: DamageType::Partial,
        }
    }
    pub fn add_input_region(&mut self, x: f32, y: f32, width: f32, height: f32) {
        add_region(&mut self.input_regions, x, y, width, height);
    }
    pub fn push(&mut self, x: f32, y: f32, width: f32, height: f32) {
        add_region(&mut self.damage_regions, x, y, width, height);
    }
    pub fn damage_type(&self) -> DamageType {
        self.damage_type
    }
    pub fn flush(&mut self) {
        self.damage_type = DamageType::Partial;
        self.input_regions.clear();
        self.damage_regions.clear();
    }
    pub fn report_input(&self) -> &[Region] {
        &self.input_regions
    }
    pub fn report_damage(&self) -> &[Region] {
        &self.damage_regions
    }
    pub fn draw_ellipse(&mut self, x: f32, y: f32, width: f32, height: f32, style: &Style) {
        if self.damage_type != DamageType::Resize {
            // from https://github.com/ritobanrc/p5-rs/src/backend/raqote.rs
            let arc = lyon_geom::Arc {
                center: point2(x, y),
                radii: vec2(width / 2., height / 2.),
                start_angle: Angle::zero(),
                sweep_angle: Angle::two_pi(),
                x_rotation: Angle::zero(),
            };

            let mut pb = PathBuilder::new();

            let start = arc.from();
            pb.line_to(start.x, start.y);

            arc.for_each_quadratic_bezier(&mut |q| {
                pb.quad_to(q.ctrl.x, q.ctrl.y, q.to.x, q.to.y);
            });

            self.push(x, y, width, height);

            let path = pb.finish();
            match &mut self.backend {
                Backend::Raqote(dt) => fill_target(dt, &path, style),
            }
        }
    }
    pub fn draw_image(&mut self, x: f32, y: f32, image: Image) {
        if self.damage_type != DamageType::Resize {
            self.push(x, y, image.width as f32, image.height as f32);
            match &mut self.backend {
                Backend::Raqote(dt) => dt.draw_image_at(x, y, &image, &DRAW_OPTIONS),
            }
        }
    }
    pub fn draw_image_with_size(&mut self, x: f32, y: f32, image: Image, width: f32, height: f32) {
        if self.damage_type != DamageType::Resize {
            self.push(x, y, width, height);
            match &mut self.backend {
                Backend::Raqote(dt) => dt.draw_image_with_size_at(width, height, x, y, &image, &DRAW_OPTIONS),
            }
        }
    }
    pub fn draw_rectangle(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: [f32; 4],
        style: &Style,
    ) {
        if self.damage_type != DamageType::Resize {
            let mut pb = PathBuilder::new();
            let mut cursor = (x, y);

            // Sides length
            let top = width - radius[0] - radius[1];
            let right = height - radius[1] - radius[2];
            let left = height - radius[0] - radius[3];
            let bottom = width - radius[2] - radius[3];

            // Positioning the cursor
            cursor.0 += radius[0];
            cursor.1 += radius[0];

            // Drawing the outline
            pb.arc(cursor.0, cursor.1, radius[0], PI, PI / 2.);
            cursor.0 += top;
            cursor.1 -= radius[0];
            pb.line_to(cursor.0, cursor.1);
            cursor.1 += radius[1];
            pb.arc(cursor.0, cursor.1, radius[1], -PI / 2., PI / 2.);
            cursor.0 += radius[1];
            cursor.1 += right;
            pb.line_to(cursor.0, cursor.1);
            cursor.0 -= radius[2];
            pb.arc(cursor.0, cursor.1, radius[2], 0., PI / 2.);
            cursor.1 += radius[2];
            cursor.0 -= bottom;
            pb.line_to(cursor.0, cursor.1);
            cursor.1 -= radius[3];
            pb.arc(cursor.0, cursor.1, radius[3], PI / 2., PI / 2.);
            cursor.0 -= radius[3];
            cursor.1 -= left;
            pb.line_to(cursor.0, cursor.1);

            // Closing path
            pb.close();
            let path = pb.finish();
            self.push(x, y, width, height);

            match &mut self.backend {
                Backend::Raqote(dt) => fill_target(dt, &path, style),
            }
        }
    }
    pub fn resize(&mut self, width: i32, height: i32) {
        self.partial_damage();
        match &mut self.backend {
            Backend::Raqote(dt) => {
                *dt = DrawTarget::new(width, height);
                self.flush();
            }
        }
    }
    pub fn full_damage(&mut self) {
        self.damage_type = DamageType::Full;
    }
    pub fn partial_damage(&mut self) {
        match &self.damage_type {
            DamageType::Full => {}
            _ => self.damage_type = DamageType::Partial,
        }
    }
    pub fn request_resize(&mut self) {
        match &self.damage_type {
            DamageType::Partial => self.damage_type = DamageType::Resize,
            _ => {}
        }
    }
    pub fn clear(&mut self) {
        match &mut self.backend {
            Backend::Raqote(dt) => {
                dt.clear(SolidSource::from_unpremultiplied_argb(0, 0, 0, 0));
            }
        }
    }
    pub fn len(&self) -> usize {
        self.damage_regions.len()
    }
    pub fn is_damaged(&self) -> bool {
        !self.damage_regions.is_empty()
    }
    pub fn load_font(&mut self, name: &str, path: &std::path::Path) {
        if let Ok(glyph_cache) = GlyphCache::load(path) {
            self.font_cache.insert(name.to_owned(), glyph_cache);
        }
    }
    pub fn store_value<T: Any>(&mut self, name: &str, value: T) {
        if let Some(inner_value) = self.values.get_mut(name) {
            *inner_value.downcast_mut::<T>().expect("Invalid Type") = value;
        } else {
            self.values.insert(name.to_string(), Box::new(value));
        }
    }
    pub fn get_value<T: Any>(&mut self, name: &str) -> Option<&T> {
        if let Some(value) = self.values.get(name) {
            value.downcast_ref()
        } else {
            None
        }
    }
    pub fn get_fonts(&self, fonts: &[String]) -> Vec<&Font> {
        fonts
            .iter()
            .filter_map(|font| {
                if let Some(glyph_cache) = self.font_cache.get(font) {
                    return Some(&glyph_cache.font);
                }
                None
            })
            .collect()
    }
    pub fn draw_label(
        &mut self,
        x: f32,
        y: f32,
        fonts: &[String],
        glyphs: &[GlyphPosition],
        source: SolidSource,
    ) {
        if self.damage_type != DamageType::Resize {
            for gp in glyphs {
                if let Some(glyph_cache) = self.font_cache.get_mut(&fonts[gp.key.font_index as usize]) {
                    if let Some(pixmap) = glyph_cache.render_glyph(gp, source) {
                        match &mut self.backend {
                            Backend::Raqote(dt) => dt.draw_image_at(
                                x.round() + gp.x,
                                y.round() + gp.y,
                                &Image {
                                    data: &pixmap,
                                    width: gp.width as i32,
                                    height: gp.height as i32,
                                },
                                &ATOP_OPTIONS,
                            ),
                        }
                    }
                }
            }
        }
    }
}

fn fill_target(dt: &mut DrawTarget, path: &Path, style: &Style) {
    match style {
        Style::Fill(source) => {
            dt.fill(&path, &Source::Solid(*source), &DRAW_OPTIONS);
        }
        Style::Border(source, border) => {
            let stroke = StrokeStyle {
                width: *border,
                cap: LineCap::Round,
                join: LineJoin::Miter,
                miter_limit: 1.,
                dash_array: Vec::new(),
                dash_offset: 0.,
            };
            dt.stroke(&path, &Source::Solid(*source), &stroke, &ATOP_OPTIONS);
        }
        _ => {}
    }
}

impl Geometry for Context {
    fn width(&self) -> f32 {
        match &self.backend {
            Backend::Raqote(dt) => dt.width() as f32,
        }
    }
    fn height(&self) -> f32 {
        match &self.backend {
            Backend::Raqote(dt) => dt.height() as f32,
        }
    }
}

impl Drawable for Context {
    fn set_color(&mut self, color: u32) {
        let color = color.to_be_bytes();
        match &mut self.backend {
            Backend::Raqote(dt) => dt.fill_rect(
                0.,
                0.,
                dt.width() as f32,
                dt.height() as f32,
                &Source::Solid(SolidSource {
                    a: color[0],
                    r: color[1],
                    g: color[2],
                    b: color[3],
                }),
                &DrawOptions::new(),
            ),
        }
    }
    fn draw(&self, ctx: &mut Context, x: f32, y: f32) {
        ctx.damage_regions.push(Region {
            x,
            y,
            width: ctx.width(),
            height: ctx.height(),
        });
        match &mut ctx.backend {
            Backend::Raqote(dt) => match &self.backend {
                Backend::Raqote(st) => dt.blend_surface(
                    &st,
                    Box2D::new(
                        euclid::point2(x as i32, y as i32),
                        euclid::point2((x + self.width()) as i32, (y + self.height()) as i32),
                    ),
                    Point2D::new(x as i32, y as i32),
                    BlendMode::Add,
                ),
            },
        }
    }
}

impl Deref for Context {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match &self.backend {
            Backend::Raqote(dt) => dt.get_data_u8(),
        }
    }
}

impl DerefMut for Context {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.backend {
            Backend::Raqote(dt) => dt.get_data_u8_mut(),
        }
    }
}
