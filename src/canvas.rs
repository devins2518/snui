use crate::widgets::text::{Font, GlyphCache, GlyphPosition};
use crate::*;
use euclid::default::{Box2D, Point2D};
use lyon_geom::euclid::{point2, vec2, Angle};
use raqote::*;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::ops::{Deref, DerefMut};
use widgets::primitives::Style;

pub enum Backend {
    Raqote(DrawTarget),
}

const DRAW_OPTIONS: DrawOptions = DrawOptions {
    blend_mode: BlendMode::SrcOver,
    alpha: 1.,
    antialias: AntialiasMode::Gray,
};

#[derive(Debug, Copy, Clone)]
pub struct DamageReport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct Canvas {
    backend: Backend,
    damage: Vec<DamageReport>,
    font_cache: HashMap<String, GlyphCache>,
}

impl Canvas {
    pub fn new(backend: Backend) -> Self {
        Self {
            backend,
            damage: Vec::new(),
            font_cache: HashMap::new(),
        }
    }
    pub fn push(&mut self, x: f32, y: f32, width: f32, height: f32) {
        if let Some(last) = self.damage.last() {
            if last.x > x && last.y > y && last.x < x + width && last.y < y + height {
                self.damage.push(DamageReport {
                    x,
                    y,
                    width,
                    height,
                });
            }
        } else {
            self.damage.push(DamageReport {
                x,
                y,
                width,
                height,
            });
        }
    }
    pub fn flush_damage(&mut self) {
        self.damage.clear();
    }
    pub fn report(&self) -> &[DamageReport] {
        &self.damage
    }
    pub fn draw_ellipse(&mut self, x: f32, y: f32, width: f32, height: f32, style: &Style) {
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
    pub fn draw_image(&mut self, x: f32, y: f32, image: Image) {
        self.push(x, y, image.width as f32, image.height as f32);
        match &mut self.backend {
            Backend::Raqote(dt) => dt.draw_image_at(x, y, &image, &DRAW_OPTIONS),
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
    pub fn resize(&mut self, width: i32, height: i32) {
        match &mut self.backend {
            Backend::Raqote(dt) => {
                *dt = DrawTarget::new(width, height);
                self.damage.clear();
            }
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
        self.damage.len()
    }
    pub fn is_damaged(&self) -> bool {
        !self.damage.is_empty()
    }
    pub fn load_font(&mut self, name: &str, path: &std::path::Path) {
        if let Ok(glyph_cache) = GlyphCache::load(path) {
            self.font_cache.insert(name.to_owned(), glyph_cache);
        }
    }
    pub fn get_fonts(&self, fonts: &[String]) -> Vec<&Font> {
        fonts.iter().filter_map(|font| {
            if let Some(glyph_cache) = self.font_cache.get(font) {
                return Some(&glyph_cache.font)
            }
            None
        }).collect()
    }
    pub fn draw_label(
        &mut self,
        x: f32,
        y: f32,
        fonts: &[String],
        glyphs: &Vec<GlyphPosition>,
        source: SolidSource,
    ) {
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
                            &DRAW_OPTIONS,
                        ),
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
                cap: LineCap::Butt,
                join: LineJoin::Miter,
                miter_limit: 1.,
                dash_array: Vec::new(),
                dash_offset: 0.,
            };
            dt.stroke(&path, &Source::Solid(*source), &stroke, &DRAW_OPTIONS);
        }
        _ => {}
    }
}

impl Geometry for Canvas {
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

impl Drawable for Canvas {
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
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32) {
        canvas.damage.push(DamageReport {
            x,
            y,
            width: canvas.width(),
            height: canvas.height(),
        });
        match &mut canvas.backend {
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

impl Deref for Canvas {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match &self.backend {
            Backend::Raqote(dt) => dt.get_data_u8(),
        }
    }
}

impl DerefMut for Canvas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.backend {
            Backend::Raqote(dt) => dt.get_data_u8_mut(),
        }
    }
}
