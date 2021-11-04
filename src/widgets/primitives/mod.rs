pub mod shapes;

use crate::scene::*;
use crate::*;
use raqote::*;
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, Debug)]
pub enum Style {
    Fill(SolidSource),
    Border(SolidSource, f32),
    Empty,
}

#[derive(Copy, Clone, Debug)]
pub enum Shape {
    Rectangle,
    Circle,
}

pub struct WidgetShell<W: Widget> {
    child: W,
    shape: Shape,
    radius: [f32; 4],
    border: Style,
    background: Style,
    padding: [f32; 4],
    previous_region: Option<Region>,
}

impl<W: Widget> Geometry for WidgetShell<W> {
    fn width(&self) -> f32 {
        self.child.width() + self.padding[1] + self.padding[3]
    }
    fn height(&self) -> f32 {
        self.child.height() + self.padding[0] + self.padding[2]
    }
}

impl<W: Widget> Drawable for WidgetShell<W> {
    fn set_color(&mut self, color: u32) {
        self.child.set_color(color);
    }
    fn draw(&self, ctx: &mut Context, x: f32, y: f32) {
        let width = self.width();
        let height = self.height();
        match self.shape {
            Shape::Rectangle => {
                ctx.draw_rectangle(x, y, width, height, self.radius, &self.background);
                if let Style::Border(_, border_width) = &self.border {
                    ctx.draw_rectangle(
                        x + border_width / 2.,
                        y + border_width / 2.,
                        width - border_width,
                        height - border_width,
                        self.radius,
                        &self.border,
                    );
                }
            }
            Shape::Circle => {
                ctx.draw_ellipse(
                    x + width / 2.,
                    y + height / 2.,
                    width,
                    height,
                    &self.background,
                );
                if let Style::Border(_, border_width) = &self.border {
                    ctx.draw_ellipse(
                        x + width / 2. + border_width / 2.,
                        y + height / 2. + border_width / 2.,
                        width - border_width,
                        height - border_width,
                        &self.border,
                    );
                }
            }
        }
        self.child
            .draw(ctx, x + self.padding[3], y + self.padding[0]);
    }
}

impl<W: Widget> Widget for WidgetShell<W> {
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, ctx: &mut Context, dispatch: &Dispatch) {
        self.previous_region = Some(Region::new(wx, wy, self.width(), self.height()));
        match self.background {
            Style::Fill(source) => {
                ctx.add_background(Background::Color(source));
            }
            _ => {}
        }
        self.child
            .roundtrip(wx + self.padding[3], wy + self.padding[0], ctx, dispatch);
    }
}

impl<W: Widget> WidgetShell<W> {
    pub fn default(child: W) -> Self {
        WidgetShell {
            child,
            background: Style::Empty,
            border: Style::Empty,
            shape: Shape::Rectangle,
            radius: [0.; 4],
            padding: [0.; 4],
            previous_region: None,
        }
    }
    pub fn rect(child: W, padding: u32, border_width: u32, background: u32, border: u32) -> Self {
        Self {
            child,
            background: if background != 0 {
                Style::fill(background)
            } else {
                Style::Empty
            },
            border: if border != 0 {
                Style::border(border, border_width as f32)
            } else {
                Style::Empty
            },
            shape: Shape::Rectangle,
            radius: [0.; 4],
            padding: [padding as f32; 4],
            previous_region: None,
        }
    }
    pub fn circle(padding: u32, border_width: u32, background: u32, border: u32, child: W) -> Self {
        Self {
            child,
            background: if background != 0 {
                Style::fill(background)
            } else {
                Style::Empty
            },
            border: if border != 0 {
                Style::border(border, border_width as f32)
            } else {
                Style::Empty
            },
            shape: Shape::Circle,
            radius: [0.; 4],
            padding: [padding as f32; 4],
            previous_region: None,
        }
    }
    pub fn set_radius(&mut self, radius: [f32; 4]) {
        self.radius = radius;
    }
    pub fn set_border_width(&mut self, border_width: f32) {
        if let Style::Border(color, _) = &self.border {
            self.border = Style::Border(*color, border_width);
        } else {
            self.border = Style::border(0, border_width);
        }
    }
    pub fn set_border_color(&mut self, color: u32) {
        if let Style::Border(_, width) = &self.border {
            self.border = Style::border(color, *width);
        } else {
            self.border = Style::border(color, 0.);
        }
    }
    pub fn set_background(&mut self, color: u32) {
        self.background = Style::fill(color);
    }
    pub fn set_padding(&mut self, padding: [u32; 4]) {
        self.padding = [
            padding[0] as f32,
            padding[1] as f32,
            padding[2] as f32,
            padding[3] as f32,
        ];
    }
    pub fn unwrap(self) -> W {
        self.child
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
