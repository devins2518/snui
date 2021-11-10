use crate::*;
use euclid::default::{Box2D, Point2D};
use lyon_geom::euclid::{point2, vec2, Angle};
use raqote::*;
use scene::*;
use std::any::Any;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::ops::{Deref, DerefMut};
use widgets::primitives::Style;
use widgets::text::LabelData;
use widgets::text::{Font, GlyphCache};
use widgets::u32_to_source;

const ATOP_OPTIONS: DrawOptions = DrawOptions {
    alpha: 1.,
    blend_mode: BlendMode::SrcAtop,
    antialias: AntialiasMode::Gray,
};

const DRAW_OPTIONS: DrawOptions = DrawOptions {
    blend_mode: BlendMode::SrcOver,
    alpha: 1.,
    antialias: AntialiasMode::Gray,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    None,
    Partial,
    Full,
}

pub enum Backend {
    Raqote(DrawTarget),
    Dummy,
}

pub struct Context {
    backend: Backend,
    pending_damage: Vec<Region>,
    values: HashMap<String, Box<dyn Any>>,
    font_cache: HashMap<String, GlyphCache>,
}

impl Context {
    pub fn new(backend: Backend) -> Self {
        Self {
            backend,
            values: HashMap::new(),
            pending_damage: Vec::new(),
            font_cache: HashMap::new(),
        }
    }
    pub fn damage_region(&mut self, bg: &Background, region: &Region) {
        match bg {
            Background::Color(source) => match &mut self.backend {
                Backend::Raqote(dt) => dt.fill_rect(
                    region.x,
                    region.y,
                    region.width,
                    region.height,
                    &Source::Solid(*source),
                    &ATOP_OPTIONS,
                ),
                _ => {}
            },
            Background::Transparent => match &mut self.backend {
                Backend::Raqote(dt) => dt.fill_rect(
                    region.x,
                    region.y,
                    region.width,
                    region.height,
                    &Source::Solid(u32_to_source(0)),
                    &DrawOptions {
                        alpha: 1.,
                        antialias: AntialiasMode::Gray,
                        blend_mode: BlendMode::SrcIn
                    },
                ),
                _ => {}
            },
        }
        self.pending_damage.push(*region);
    }
    pub fn add_region(&mut self, region: Region) {
        self.pending_damage.push(region);
    }
    pub fn resize(&mut self, width: i32, height: i32) {
        match &mut self.backend {
            Backend::Raqote(dt) => {
                *dt = DrawTarget::new(width, height);
                self.flush();
            }
            _ => {}
        }
    }
    pub fn clear(&mut self) {
        match &mut self.backend {
            Backend::Raqote(dt) => {
                dt.clear(SolidSource::from_unpremultiplied_argb(0, 0, 0, 0));
            }
            _ => {}
        }
        self.flush();
    }
    pub fn len(&self) -> usize {
        self.pending_damage.len()
    }
    pub fn is_damaged(&self) -> bool {
        !self.pending_damage.is_empty()
    }
    pub fn flush(&mut self) {
        self.pending_damage.clear();
    }
    pub fn report_damage(&self) -> &[Region] {
        &self.pending_damage
    }
    pub fn draw_image(&mut self, x: f32, y: f32, image: Image) {
        match &mut self.backend {
            Backend::Raqote(dt) => dt.draw_image_at(x, y, &image, &DRAW_OPTIONS),
            _ => {}
        }
    }
    pub fn draw_image_with_size(&mut self, x: f32, y: f32, image: Image, width: f32, height: f32) {
        match &mut self.backend {
            Backend::Raqote(dt) => {
                dt.draw_image_with_size_at(width, height, x, y, &image, &DRAW_OPTIONS)
            }
            _ => {}
        }
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

        let path = pb.finish();
        match &mut self.backend {
            Backend::Raqote(dt) => fill_target(dt, &path, style),
            _ => {}
        }
    }
    pub fn draw_rectangle(
        &mut self,
        x: f32,
        y: f32,
        mut width: f32,
        mut height: f32,
        radius: [f32; 4],
        style: &Style,
    ) {
        let mut pb = PathBuilder::new();
        let mut cursor = match style {
            Style::Border(_, border) => {
                width -= border;
                height -= border;
                (x + border/2., y + border/2.)
            }
            _ => (x, y)
        };

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

        match &mut self.backend {
            Backend::Raqote(dt) => fill_target(dt, &path, style),
            _ => {}
        }
    }
    pub fn draw_label(&mut self, x: f32, y: f32, data: &LabelData) {
        let fonts = &data.fonts;
        let source = data.source;
        for gp in &data.glyphs {
            if let Some(glyph_cache) = self.font_cache.get_mut(&fonts[gp.key.font_index as usize]) {
                if let Some(pixmap) = glyph_cache.render_glyph(&gp, source) {
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
                        _ => {}
                    }
                }
            }
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
            _ => 0.
        }
    }
    fn height(&self) -> f32 {
        match &self.backend {
            Backend::Raqote(dt) => dt.height() as f32,
            _ => 0.
        }
    }
}

impl Deref for Context {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match &self.backend {
            Backend::Raqote(dt) => dt.get_data_u8(),
            _ => panic!("Dummy backend cannot return a slice reference")
        }
    }
}

impl DerefMut for Context {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.backend {
            Backend::Raqote(dt) => dt.get_data_u8_mut(),
            _ => panic!("Dummy backend cannot return a slice reference")
        }
    }
}
