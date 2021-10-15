use crate::*;
use raqote::*;
use crate::widgets::primitives::*;

impl Style {
    pub fn fill(color: u32) -> Self {
        let color = color.to_be_bytes();
        Style::Fill(SolidSource {
            a: color[0],
            r: color[1],
            g: color[2],
            b: color[3],
        })
    }
    pub fn border(color: u32, size: f32) -> Self {
        let color = color.to_be_bytes();
        Style::Border(
            SolidSource {
                a: color[0],
                r: color[1],
                g: color[2],
                b: color[3],
            },
            size,
        )
    }
    pub fn is_empty(&self) -> bool {
        if let Style::Empty = self {
            true
        } else {
            false
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    damaged: bool,
    width: f32,
    height: f32,
    style: Style,
    // (tl, tr, br, bl)
    radius: [f32; 4],
}

impl Rectangle {
    pub fn new(width: f32, height: f32, style: Style) -> Self {
        Rectangle {
            damaged: true,
            width,
            height,
            style,
            radius: [0.; 4],
        }
    }
    pub fn square(size: f32, style: Style) -> Self {
        Rectangle {
            damaged: true,
            width: size,
            height: size,
            style,
            radius: [0.; 4],
        }
    }
    pub fn set_radius(&mut self, radius: [f32; 4]) {
        self.radius = radius;
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

impl Drawable for Rectangle {
    fn set_color(&mut self, color: u32) {
        let color = color.to_be_bytes();
        if let Style::Border(source, _) = &mut self.style {
            *source = SolidSource {
                a: color[0],
                r: color[1],
                g: color[2],
                b: color[3],
            };
        } else if let Style::Fill(source) = &mut self.style {
            *source = SolidSource {
                a: color[0],
                r: color[1],
                g: color[2],
                b: color[3],
            };
        }
    }
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32) {
        if !self.style.is_empty() && self.damaged {
            canvas.draw_rectangle(
                x,
                y,
                self.width(),
                self.height(),
                self.radius,
                &self.style,
            );
        }
    }
}

impl Widget for Rectangle {
    fn roundtrip<'d>(&'d mut self, _wx: f32, _wy: f32, dispatch: &Dispatch) -> Option<Damage> {
        if let Dispatch::Commit = dispatch {
            self.damaged = self.damaged == false;
        }
        None
    }
    fn damaged(&self) -> bool {
        self.damaged
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Circle {
    damaged: bool,
    style: Style,
    // (tl, tr, br, bl)
    radius: f32,
}

impl Circle {
    pub fn new(radius: f32, style: Style) -> Self {
        Circle {
            damaged: true,
            style,
            radius: radius,
        }
    }
}

impl Geometry for Circle {
    fn width(&self) -> f32 {
        self.radius * 2.
    }
    fn height(&self) -> f32 {
        self.radius * 2.
    }
}

impl Drawable for Circle {
    fn set_color(&mut self, color: u32) {
        let color = color.to_be_bytes();
        if let Style::Border(source, _) = &mut self.style {
            *source = SolidSource {
                a: color[0],
                r: color[1],
                g: color[2],
                b: color[3],
            };
        } else if let Style::Fill(source) = &mut self.style {
            *source = SolidSource {
                a: color[0],
                r: color[1],
                g: color[2],
                b: color[3],
            };
        }
    }
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32) {
        if !self.style.is_empty() && self.damaged() {
            canvas.draw_ellipse(
                x,
                y,
                self.width(),
                self.height(),
                &self.style
            );
        }
    }
}

impl Widget for Circle {
    fn roundtrip<'d>(&'d mut self, _wx: f32, _wy: f32, dispatch: &Dispatch) -> Option<Damage> {
        if let Dispatch::Commit = dispatch {
            self.damaged = self.damaged == false;
        }
        None
    }
    fn damaged(&self) -> bool {
        self.damaged
    }
}
