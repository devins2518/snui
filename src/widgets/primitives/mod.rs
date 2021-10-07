pub mod shapes;

use crate::*;
use raqote::*;
use shapes::*;

#[derive(Copy, Clone, Debug)]
pub enum Style {
    Fill(SolidSource),
    Border(SolidSource, f32),
    Empty
}

pub enum Shape {
    Rectangle,
    Circle,
    Triangle
}

pub struct Boxed<W: Widget> {
    pub child: W,
    damaged: bool,
    shape: Shape,
    radius: [f32;4],
    border_color: Style,
    background_color: Style,
    border_width: u32,
    padding: [u32; 4],
}

impl<W: Widget> Geometry for Boxed<W> {
    fn width(&self) -> u32 {
        self.child.width() + (self.border_width * 2) + self.padding[1] + self.padding[3]
    }
    fn height(&self) -> u32 {
        self.child.height() + (self.border_width * 2) + self.padding[0] + self.padding[2]
    }
}

impl<W: Widget> Drawable for Boxed<W> {
    fn set_color(&mut self, color: u32) {
        self.background_color = Style::fill(color);
    }
    fn draw(&self, canvas: &mut Canvas, x: u32, y: u32) {
        match self.shape {
            Shape::Rectangle => {
                let width = self.child.width() + self.padding[1] + self.padding[3];
                let height = self.child.height() + self.padding[0] + self.padding[2];
                let mut border =  Rectangle::new(width as f32, height as f32, self.border_color);
                border.set_radius(self.radius);
                border.draw(canvas, x, y);
            }
            _ => {}
        }
        self.child
            .draw(canvas, x + self.padding[3] + self.border_width/2, y + self.padding[0] + self.border_width/2);
    }
}

impl<W: Widget> Widget for Boxed<W> {
    fn damaged(&self) -> bool {
        self.damaged
    }
    fn roundtrip<'d>(
        &'d mut self,
        widget_x: u32,
        widget_y: u32,
        dispatched: &Dispatch,
    ) -> Option<Damage> {
        if let Dispatch::Commit = dispatched {
            self.damaged = self.damaged == false;
        }
        self.child.roundtrip(
            widget_x + self.padding[3] + self.border_width,
            widget_y + self.padding[0] + self.border_width,
            dispatched,
        )
    }
}

impl<W: Widget> Boxed<W> {
    pub fn rect(padding: u32, border_width: u32, background_color: u32, border_color: u32, child: W) -> Self {
        Self {
            child,
            background_color: if background_color != 0 {
                Style::fill(background_color)
            } else { Style::Empty },
            border_color: if border_color != 0 {
                Style::border(border_color, border_width as f32 * 2.)
            } else { Style::Empty },
            border_width,
            shape: Shape::Rectangle,
            radius: [0.;4],
            padding: [padding; 4],
            damaged: true,
        }
    }
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = [radius; 4];
    }
    pub fn set_padding(&mut self, padding: u32) {
        self.padding = [padding; 4];
    }
}
