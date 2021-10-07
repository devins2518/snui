use crate::*;
use crate::widgets::primitives::*;
use raqote::*;

const PI: f32 = 3.14159;

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
        Style::Border(SolidSource {
            a: color[0],
            r: color[1],
            g: color[2],
            b: color[3],
        }, size)
    }
}

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
            radius: [0.;4]
        }
    }
    pub fn square(size: f32, style: Style) -> Self {
        Rectangle {
            damaged: true,
            width: size,
            height: size,
            style,
            radius: [0.;4]
        }
    }
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = [radius; 4];
    }
}

impl Geometry for Rectangle {
    fn width(&self) -> u32 {
        self.width as u32
    }
    fn height(&self) -> u32 {
        self.height as u32
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
    fn draw(&self, canvas: &mut Canvas, x: u32, y: u32) {
        let dt = canvas.target();
        let mut pb = PathBuilder::new();
        let mut cursor = (x as f32, y as f32);

		// Sides length
        let top = self.width - self.radius[0] - self.radius[1];
        let right = self.height - self.radius[1] - self.radius[2];
        let left = self.height - self.radius[0] - self.radius[3];
        let bottom = self.width - self.radius[2] - self.radius[3];

		// Positioning the cursor
		cursor.0 += self.radius[0];
		cursor.1 += self.radius[0];

        // Drawing the outline
        pb.arc(cursor.0, cursor.1, self.radius[0], PI, PI/2.);
		cursor.0 += top;
		cursor.1 -= self.radius[0];
        pb.line_to(cursor.0, cursor.1);
		cursor.1 += self.radius[1];
        pb.arc(cursor.0, cursor.1, self.radius[1], -PI/2., PI/2.);
		cursor.0 += self.radius[1];
		cursor.1 += right;
        pb.line_to(cursor.0, cursor.1);
        cursor.0 -= self.radius[2];
        pb.arc(cursor.0, cursor.1, self.radius[2], 0., PI/2.);
		cursor.1 += self.radius[2];
		cursor.0 -= bottom;
        pb.line_to(cursor.0, cursor.1);
		cursor.1 -= self.radius[3];
        pb.arc(cursor.0, cursor.1, self.radius[3], PI/2., PI/2.);
		cursor.0 -= self.radius[3];
		cursor.1 -= left;
        pb.line_to(cursor.0, cursor.1);

        // Closing path
        pb.close();
        let path = pb.finish();

        match &self.style {
            Style::Fill(source) => {
                dt.fill(&path, &Source::Solid(*source), &DrawOptions::new());
            }
            Style::Border(source, border) => {
                let stroke = StrokeStyle {
                    width: *border,
                    cap: LineCap::Butt,
                    join: LineJoin::Miter,
                    miter_limit: 10.,
                    dash_array: Vec::new(),
                    dash_offset: 0.,
                };
                dt.stroke(
                    &path,
                    &Source::Solid(*source),
                    &stroke,
                    &DrawOptions::new()
                );
            }
            Style::Empty => {}
        }
    }
}

impl Widget for Rectangle {
    fn roundtrip<'d>(
        &'d mut self,
        _widget_x: u32,
        _widget_y: u32,
        dispatched: &Dispatch,
    ) -> Option<Damage> {
        if let Dispatch::Commit = dispatched {
            self.damaged = self.damaged == false;
        }
        None
    }
    fn damaged(&self) -> bool {
        self.damaged
    }
}
