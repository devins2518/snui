use raqote::*;
use crate::*;
use std::f32::consts::PI;
use widgets::primitives::Style;
use std::ops::{Deref, DerefMut};
use euclid::default::{Box2D, Point2D};
use lyon_geom::euclid::{point2, vec2, Angle};

pub enum Backend<'b> {
    Raqote(&'b mut DrawTarget)
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
    // container: bool,
}

pub struct Canvas<'b> {
    backend: Backend<'b>,
    damage: Vec<DamageReport>,
    // TO-DO
    // font_cache: FontCache
}

impl<'b> Canvas<'b> {
    pub fn new(backend: Backend<'b>) -> Self {
        Self {
            backend,
            damage: Vec::new()
        }
    }
    pub fn push(&mut self, x: f32, y: f32, width: f32, height: f32) {
        if let Some(last) = self.damage.last() {
            if last.x > x
               && last.y > y
               && last.x < x + width
               && last.y < y + height
            {
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
    pub fn clear(&mut self) {
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
            _ => {}
        }
    }
    pub fn draw_image(&mut self, x: f32, y: f32, image: Image) {
        self.push(x, y, image.width as f32, image.height as f32);
        match &mut self.backend {
            Backend::Raqote(dt) => {
                dt.draw_image_at(
                    x,
                    y,
                    &image,
                    &DRAW_OPTIONS
                )
            }
            _ => {}
        }
    }
    pub fn draw_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32, radius: [f32; 4], style: &Style) {
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
                cap: LineCap::Round,
                join: LineJoin::Miter,
                miter_limit: 10.,
                dash_array: Vec::new(),
                dash_offset: 0.,
            };
            dt.stroke(&path, &Source::Solid(*source), &stroke, &DRAW_OPTIONS);
        }
        _ => {}
    }
}

impl<'b> Geometry for Canvas<'b> {
    fn width(&self) -> f32 {
        match &self.backend {
            Backend::Raqote(dt) => {
                dt.width() as f32
            }
            _ => 0.
        }
    }
    fn height(&self) -> f32 {
        match &self.backend {
            Backend::Raqote(dt) => {
                dt.height() as f32
            }
            _ => 0.
        }
    }
}

impl<'b> Drawable for Canvas<'b> {
    fn set_color(&mut self, color: u32) {
        let color = color.to_be_bytes();
        match &mut self.backend {
            Backend::Raqote(dt) => {
                dt.fill_rect(
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
                )
            }
        }
    }
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32) {
        for d in &self.damage {
            canvas.damage.push(*d);
        }
        match &mut canvas.backend {
            Backend::Raqote(dt) => {
                match &self.backend {
                    Backend::Raqote(st) => {
                        dt.blend_surface(
                            &st,
                            Box2D::new(
                                euclid::point2(x as i32, y as i32),
                                euclid::point2((x + self.width()) as i32, (y + self.height()) as i32),
                            ),
                            Point2D::new(x as i32, y as i32),
                            BlendMode::Add,
                        )
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl<'b> Deref for Canvas<'b> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match &self.backend {
            Backend::Raqote(dt) => {
                dt.get_data_u8()
            }
        }
    }
}

impl<'b> DerefMut for Canvas<'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.backend {
            Backend::Raqote(dt) => {
                dt.get_data_u8_mut()
            }
        }
    }
}

