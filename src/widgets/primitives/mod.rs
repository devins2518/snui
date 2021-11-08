pub mod shapes;

use crate::scene::*;
use crate::widgets::u32_to_source;
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
    padding: [f32; 4],
    border: Style,
    damage: bool,
    background: Background,
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
        self.damage = true;
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
                    &self.background.into_style(),
                );
                if let Style::Border(_, border_width) = &self.border {
                    border = *border_width;
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
                    &self.background.into_style(),
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
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, ctx: &mut Context, dispatch: &Dispatch) {
        let bg = ctx.get_background();
        let border = if let Style::Border(_, border) = self.border {
            border
        } else {
            0.
        };
        ctx.update_scene(
            Region::new(wx, wy, self.width(), self.height()),
            self.background.clone(),
        );
        if self.damage {
            self.damage = false;
            ctx.force_damage();
        }
        self.child.roundtrip(
            wx + self.padding[3] + border,
            wy + self.padding[0] + border,
            ctx,
            dispatch,
        );
        ctx.update_scene(Region::new(wx, wy, self.width(), self.height()), bg);
    }
}

impl<W: Widget> WidgetShell<W> {
    pub fn default(child: W) -> Self {
        WidgetShell {
            child,
            damage: false,
            background: Background::Transparent,
            border: Style::Empty,
            shape: Shape::Rectangle,
            radius: [0.; 4],
            padding: [0.; 4],
        }
    }
    pub fn rect(child: W, padding: u32, border_width: u32, background: u32, border: u32) -> Self {
        Self {
            child,
            damage: false,
            background: if background != 0 {
                Background::Color(u32_to_source(background))
            } else {
                Background::Transparent
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
            damage: true,
            background: if background != 0 {
                Background::Color(u32_to_source(background))
            } else {
                Background::Transparent
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
        self.damage = true;
        self.radius = radius;
    }
    pub fn set_border_width(&mut self, border_width: f32) {
        self.damage = true;
        if let Style::Border(color, _) = &self.border {
            self.border = Style::Border(*color, border_width);
        } else {
            self.border = Style::border(0, border_width);
        }
    }
    pub fn set_border_color(&mut self, color: u32) {
        self.damage = true;
        if let Style::Border(_, width) = &self.border {
            self.border = Style::border(color, *width);
        } else {
            self.border = Style::border(color, 0.);
        }
    }
    pub fn set_background(&mut self, color: u32) {
        self.damage = true;
        self.background = Background::Color(u32_to_source(color));
    }
    pub fn set_padding(&mut self, padding: [f32; 4]) {
        self.damage = true;
        self.padding = padding;
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
