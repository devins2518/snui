pub mod shapes;

use crate::scene::*;
use crate::*;
use shapes::*;
use raqote::*;
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, Debug,PartialEq)]
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
    padding: [f32; 4],
    border: Style,
    background: Style,
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
        }
    }
    pub fn rect(child: W, padding: u32, border_width: u32, background: u32, border: u32) -> Self {
        Self {
            child,
            background: if background != 0 {
                Style::fill(border)
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
    pub fn set_padding(&mut self, padding: [f32; 4]) {
        self.padding = padding;
    }
    pub fn unwrap(self) -> W {
        self.child
    }
}

impl<W: Widget> Geometry for WidgetShell<W> {
    fn width(&self) -> f32 {
        self.child.width()
            + self.padding[1]
            + self.padding[3]
            + if let Style::Border(_, border) = &self.border {
                2. * *border
            } else {
                0.
            }
    }
    fn height(&self) -> f32 {
        self.child.height()
            + self.padding[0]
            + self.padding[2]
            + if let Style::Border(_, border) = &self.border {
                2. * *border
            } else {
                0.
            }
    }
}

impl<W: Widget> Drawable for WidgetShell<W> {
    fn set_color(&mut self, color: u32) {
        self.child.set_color(color);
    }
    fn draw(&self, ctx: &mut Context, x: f32, y: f32) {
        let width = self.width();
        let height = self.height();
        let mut border = 0.;
        match self.shape {
            Shape::Rectangle => {
                ctx.draw_rectangle(
                    x,
                    y,
                    width,
                    height,
                    self.radius,
                    &self.background,
                );
                if let Style::Border(_, border_width) = &self.border {
                    border = *border_width;
                    ctx.draw_rectangle(
                        x,
                        y,
                        width,
                        height,
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
                    border = *border_width;
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
        self.child.draw(
            ctx,
            x + self.padding[3] + border,
            y + self.padding[0] + border,
        );
    }
}

impl<W: Widget> Widget for WidgetShell<W> {
    fn create_node(&self, x: f32, y: f32) -> RenderNode {
        let border_width = if let Style::Border(_, border) = self.border {
            border
        } else {
            0.
        };
        let background = Rectangle {
            width: self.width(),
            height: self.height(),
            style: self.background,
            radius: self.radius
        };
        let border= Rectangle {
            width: self.width(),
            height: self.height(),
            style: self.border,
            radius: self.radius
        };
        RenderNode::Extension {
            background: Damage::from_rectangle(x, y, background),
            border: Damage::from_rectangle(x, y, border),
            node: Box::new(self.child.create_node(x + self.padding[3] + border_width, y + self.padding[0] + border_width))
        }
    }
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, ctx: &mut Context, dispatch: &Dispatch) {
        let border = if let Style::Border(_, border) = self.border {
            border
        } else {
            0.
        };
        self.child.roundtrip(
            wx + self.padding[3] + border,
            wy + self.padding[0] + border,
            ctx,
            dispatch,
        );
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
