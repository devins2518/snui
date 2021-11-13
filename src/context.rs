use crate::*;
use lyon_geom::euclid::{point2, vec2, Angle};
use raqote::*;
use scene::*;
use std::ops::{Deref, DerefMut};
use widgets::font::*;
use widgets::shapes::Style;
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

pub enum Backend {
    Raqote(DrawTarget),
    Dummy,
}

pub struct Context {
    pub backend: Backend,
    pub font_cache: FontCache,
    pending_damage: Vec<Region>,
}

impl Context {
    pub fn new(backend: Backend) -> Self {
        Self {
            backend,
            pending_damage: Vec::new(),
            font_cache: FontCache::new(),
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
                        blend_mode: BlendMode::SrcIn,
                    },
                ),
                _ => {}
            },
        }
        self.pending_damage.push(*region);
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
    pub fn is_damaged(&self) -> bool {
        !self.pending_damage.is_empty()
    }
    pub fn flush(&mut self) {
        self.pending_damage.clear();
        self.font_cache.layouts.clear();
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
    pub fn draw_path(&mut self, path: Path, style: &Style) {
        match &mut self.backend {
            Backend::Raqote(dt) => fill_target(dt, &path, style),
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

impl Geometry for Context {
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        match &mut self.backend {
            Backend::Raqote(dt) => {
                *dt = DrawTarget::new(width as i32, height as i32);
            }
            _ => {}
        }
        Ok(())
    }
    fn width(&self) -> f32 {
        match &self.backend {
            Backend::Raqote(dt) => dt.width() as f32,
            _ => 0.,
        }
    }
    fn height(&self) -> f32 {
        match &self.backend {
            Backend::Raqote(dt) => dt.height() as f32,
            _ => 0.,
        }
    }
}

impl Deref for Context {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match &self.backend {
            Backend::Raqote(dt) => dt.get_data_u8(),
            _ => panic!("Dummy backend cannot return a slice"),
        }
    }
}

impl DerefMut for Context {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.backend {
            Backend::Raqote(dt) => dt.get_data_u8_mut(),
            _ => panic!("Dummy backend cannot return a slice"),
        }
    }
}
