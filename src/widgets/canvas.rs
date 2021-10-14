use raqote::*;
use crate::*;
use euclid::default::{Box2D, Point2D};

enum Backend<'b> {
    Raqote(&'b mut DrawTarget)
}

#[derive(Debug, Copy, Clone)]
pub struct DamageReport {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    container: bool,
}

pub struct Canvas<'b> {
    target: Backend<'b>,
    damage: Vec<DamageReport>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            target: DrawTarget::new(width as i32, height as i32),
            damage: Vec::new(),
        }
    }
    pub fn target(&mut self) -> &mut DrawTarget {
        &mut self.target
    }
    pub fn push<W: Geometry>(&mut self, x: f32, y: f32, widget: &W, container: bool) {
        if let Some(last) = self.damage.last() {
            if last.container
                && last.x > x
                && last.y > y
                && last.x < x + widget.width()
                && last.y < y + widget.height()
            {
                self.damage.push(DamageReport {
                    x,
                    y,
                    container,
                    width: widget.width(),
                    height: widget.height(),
                });
            }
        } else {
            self.damage.push(DamageReport {
                x,
                y,
                container,
                width: widget.width(),
                height: widget.height(),
            });
        }
    }
    pub fn clear(&mut self) {
        self.damage.clear();
    }
    pub fn report(&mut self) -> &[DamageReport] {
        &self.damage
    }
}

impl Geometry for Canvas {
    fn width(&self) -> f32 {
        self.target.width() as f32
    }
    fn height(&self) -> f32 {
        self.target.height() as f32
    }
}

impl Drawable for Canvas {
    fn set_color(&mut self, color: u32) {
        let color = color.to_be_bytes();
        self.target.fill_rect(
            0.,
            0.,
            self.target.width() as f32,
            self.target.height() as f32,
            &Source::Solid(SolidSource {
                a: color[0],
                r: color[1],
                g: color[2],
                b: color[3],
            }),
            &DrawOptions::new(),
        )
    }
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32) {
        for damage in &self.damage {
            canvas.damage.push(*damage);
        }
        canvas.target().blend_surface(
            &self.target,
            Box2D::new(
                euclid::point2(x as i32, y as i32),
                euclid::point2((x + self.width()) as i32, (y + self.height()) as i32),
            ),
            Point2D::new(x as i32, y as i32),
            BlendMode::Add,
        )
    }
}

impl Deref for Canvas {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.target.get_data_u8()
    }
}

impl DerefMut for Canvas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.target.get_data_u8_mut()
    }
}

