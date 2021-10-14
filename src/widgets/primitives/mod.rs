pub mod shapes;

use crate::*;
use raqote::*;
use shapes::*;

#[derive(Copy, Clone, Debug)]
pub enum Style {
    Fill(SolidSource),
    Border(SolidSource, f32),
    Empty,
}

pub enum Shape {
    Rectangle,
    Circle,
    Triangle,
}

pub struct WidgetShell<W: Widget> {
    child: W,
    damaged: bool,
    shape: Shape,
    radius: [f32; 4],
    border_color: Style,
    background_color: Style,
    border_width: f32,
    padding: [f32; 4],
}

impl<W: Widget> Geometry for WidgetShell<W> {
    fn width(&self) -> f32 {
        match self.shape {
            Shape::Circle => {
                let diameter = if self.child.width() > self.child.height() {
                    self.child.width()
                } else {
                    self.child.height()
                };
                diameter + (self.border_width * 2.) + self.padding[1] + self.padding[3]
            }
            Shape::Rectangle => {
                self.child.width() + (self.border_width * 2.) + self.padding[1] + self.padding[3]
            }
            Shape::Triangle => self.child.width(),
        }
    }
    fn height(&self) -> f32 {
        match self.shape {
            Shape::Circle => {
                let diameter = if self.child.width() > self.child.height() {
                    self.child.width()
                } else {
                    self.child.height()
                };
                diameter + (self.border_width * 2.) + self.padding[1] + self.padding[3]
            }
            Shape::Rectangle => {
                self.child.height() + (self.border_width * 2.) + self.padding[0] + self.padding[2]
            }
            Shape::Triangle => self.child.height(),
        }
    }
}

impl<W: Widget> Drawable for WidgetShell<W> {
    fn set_color(&mut self, color: u32) {
        self.background_color = Style::fill(color);
    }
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32) {
        if self.damaged {
            let width = self.child.width() + self.padding[1] + self.padding[3];
            let height = self.child.height() + self.padding[0] + self.padding[2];
            match self.shape {
                Shape::Rectangle => {
                    let mut background =
                        Rectangle::new(width as f32, height as f32, self.background_color);
                    background.set_radius(self.radius);
                    background.draw(canvas, x + self.border_width, y + self.border_width);
                    let mut border = Rectangle::new(width as f32, height as f32, self.border_color);
                    border.set_radius(self.radius);
                    border.draw(
                        canvas,
                        x + self.border_width / 2.,
                        y + self.border_width / 2.,
                    );
                }
                Shape::Circle => {
                    let radius = if width > height {
                        width as f32
                    } else {
                        height as f32
                    };
                    Circle::new(radius, self.background_color).draw(
                        canvas,
                        x + self.border_width,
                        y + self.border_width,
                    );
                    Circle::new(radius, self.border_color).draw(canvas, x, y);
                }
                _ => {}
            }
            self.child.draw(
                canvas,
                x + self.padding[3] + self.border_width / 2. + self.border_width % 2.,
                y + self.padding[0] + self.border_width / 2. + self.border_width % 2.,
            );
            canvas.push(x, y, self, true);
        } else {
            self.child.draw(canvas, x, y);
        }
    }
}

impl<W: Widget> Widget for WidgetShell<W> {
    fn damaged(&self) -> bool {
        self.child.damaged()
    }
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, dispatch: &Dispatch) -> Option<Damage> {
        if let Dispatch::Commit = dispatch {
            self.damaged = self.damaged == false;
        }
        self.child.roundtrip(
            wx + self.padding[3] + self.border_width / 2. + self.border_width % 2.,
            wy + self.padding[0] + self.border_width / 2. + self.border_width % 2.,
            dispatch,
        )
    }
}

impl<W: Widget> WidgetShell<W> {
    pub fn default(child: W) -> Self {
        WidgetShell {
            child,
            damaged: true,
            background_color: Style::Empty,
            border_color: Style::Empty,
            border_width: 0.,
            shape: Shape::Rectangle,
            radius: [0.; 4],
            padding: [0.; 4],
        }
    }
    pub fn rect(
        child: W,
        padding: u32,
        border_width: u32,
        background_color: u32,
        border_color: u32,
    ) -> Self {
        Self {
            child,
            background_color: if background_color != 0 {
                Style::fill(background_color)
            } else {
                Style::Empty
            },
            border_color: if border_color != 0 {
                Style::border(border_color, border_width as f32)
            } else {
                Style::Empty
            },
            border_width: border_width as f32,
            shape: Shape::Rectangle,
            radius: [0.; 4],
            padding: [padding as f32; 4],
            damaged: true,
        }
    }
    pub fn circle(
        padding: u32,
        border_width: u32,
        background_color: u32,
        border_color: u32,
        child: W,
    ) -> Self {
        Self {
            child,
            background_color: if background_color != 0 {
                Style::fill(background_color)
            } else {
                Style::Empty
            },
            border_color: if border_color != 0 {
                Style::border(border_color, border_width as f32)
            } else {
                Style::Empty
            },
            border_width: border_width as f32,
            shape: Shape::Circle,
            radius: [0.; 4],
            padding: [padding as f32; 4],
            damaged: true,
        }
    }
    pub fn set_radius(&mut self, radius: [f32; 4]) {
        self.radius = radius;
    }
    pub fn set_border_width(&mut self, border_width: f32) {
        self.border_width = border_width;
        if let Style::Border(color, _) = &self.border_color {
            self.border_color = Style::Border(*color, border_width);
        } else {
            self.border_color = Style::border(0, border_width);
        }
    }
    pub fn set_border_color(&mut self, color: u32) {
        if let Style::Border(_, width) = &self.border_color {
            self.border_color = Style::border(color, *width);
        } else {
            self.border_color = Style::border(color, 0.);
        }
    }
    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = Style::fill(color);
    }
    pub fn set_padding(&mut self, padding: [u32; 4]) {
        self.padding = [
            padding[0] as f32,
            padding[1] as f32,
            padding[2] as f32,
            padding[3] as f32,
        ];
    }
}

impl<W: Widget> Deref for WidgetShell<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl<W: Widget> DerefMut for WidgetShell<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}
