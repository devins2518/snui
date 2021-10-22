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
    width: f32,
    height: f32,
    style: Style,
    // (tl, tr, br, bl)
    radius: [f32; 4],
}

impl Rectangle {
    pub fn new(width: f32, height: f32, style: Style) -> Self {
        Rectangle {
            width,
            height,
            style,
            radius: [0.; 4],
        }
    }
    pub fn square(size: f32, style: Style) -> Self {
        Rectangle {
            width: size,
            height: size,
            style,
            radius: [0.; 4],
        }
    }
    pub fn empty(width: f32, height: f32) -> Self {
        Rectangle {
            width,
            height,
            style: Style::Empty,
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
        if !self.style.is_empty() {
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
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, canvas: &mut Canvas, dispatch: &Dispatch) {
        if let Dispatch::Commit = dispatch {
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Circle {
    style: Style,
    // (tl, tr, br, bl)
    radius: f32,
}

impl Circle {
    pub fn new(radius: f32, style: Style) -> Self {
        Circle {
            style,
            radius,
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
        if !self.style.is_empty() {
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
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, canvas: &mut Canvas, dispatch: &Dispatch) {
        if let Dispatch::Commit = dispatch {
        }
    }
}
