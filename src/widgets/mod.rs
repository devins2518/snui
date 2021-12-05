pub mod button;
pub mod container;
pub mod image;
pub mod shapes;
pub mod slider;
pub mod text;

use crate::scene::Coords;
pub use crate::widgets::image::Image;
use crate::*;
pub use button::Button;
pub use container::layout::WidgetLayout;
pub use shapes::Style;
use std::ops::{Deref, DerefMut};
use tiny_skia::*;

pub const START: Alignment = Alignment::Start;
pub const CENTER: Alignment = Alignment::Center;
pub const END: Alignment = Alignment::End;

pub fn u32_to_source(color: u32) -> Color {
    let color = color.to_be_bytes();
    Color::from_rgba8(color[3], color[2], color[1], color[0])
}

pub fn blend(pix_a: &Color, pix_b: &Color, t: f32) -> Color {
    let (r_a, g_a, b_a, a_a) = (pix_a.red(), pix_a.green(), pix_a.blue(), pix_a.alpha());
    let (r_b, g_b, b_b, a_b) = (pix_b.red(), pix_b.green(), pix_b.blue(), pix_b.alpha());
    let red = blend_f32(r_a, r_b, t);
    let green = blend_f32(g_a, g_b, t);
    let blue = blend_f32(b_a, b_b, t);
    let alpha = blend_f32(a_a, a_b, t);

    Color::from_rgba(red, green, blue, alpha).unwrap()
}

fn blend_f32(a: f32, b: f32, r: f32) -> f32 {
    a + ((b - a) * r)
}

#[derive(Copy, Clone, Debug)]
pub enum Alignment {
    Start,
    Center,
    End,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Constraint {
    // Size remains the same regardless
    Fixed,
    // The size is the maximum size the widget can take
    Upward,
    // The size is the minimum size the widget can take
    Downward,
}

pub struct WidgetBox<W: Widget> {
    child: W,
    coords: Coords,
    size: (f32, f32),
    constraint: Constraint,
    anchor: (Alignment, Alignment),
}

impl<W: Widget> Geometry for WidgetBox<W> {
    fn width(&self) -> f32 {
        match &self.constraint {
            Constraint::Fixed => self.size.0,
            Constraint::Upward => self.child.width().min(self.size.0),
            Constraint::Downward => self.child.width().max(self.size.0),
        }
    }
    fn height(&self) -> f32 {
        match &self.constraint {
            Constraint::Fixed => self.size.1,
            Constraint::Upward => self.child.height().min(self.size.1),
            Constraint::Downward => self.child.height().max(self.size.1),
        }
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        if width.is_sign_positive() {
            self.size.0 = width;
            return Ok(());
        }
        Err(self.size.0)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        if height.is_sign_positive() {
            self.size.1 = height;
            return Ok(());
        }
        Err(self.size.1)
    }
}

impl<W: Widget> Widget for WidgetBox<W> {
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        if let Event::Pointer(mut x, mut y, pointer) = event {
            x -= self.coords.x;
            y -= self.coords.y;
            self.child.sync(ctx, Event::Pointer(x, y, pointer));
        } else {
            self.child.sync(ctx, event);
        }
    }
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        if (self.constraint == Constraint::Fixed || self.constraint == Constraint::Upward)
            && (self.child.width() > self.size.0 || self.child.height() > self.size.1)
        {
            eprintln!(
                "WidgetBox exceeded bounds: {} x {}",
                self.width(),
                self.height()
            );
            return RenderNode::None;
        }
        let (horizontal, vertical) = &self.anchor;
        let dx = match horizontal {
            Alignment::Start => 0.,
            Alignment::Center => ((self.width() - self.child.width()) / 2.).floor(),
            Alignment::End => (self.width() - self.child.width()).floor(),
        };
        let dy = match vertical {
            Alignment::Start => 0.,
            Alignment::Center => ((self.height() - self.child.height()) / 2.).floor(),
            Alignment::End => (self.height() - self.child.height()).floor(),
        };
        self.coords = Coords::new(dx, dy);
        self.child.create_node(x + dx, y + dy)
    }
}

impl<W: Widget> WidgetBox<W> {
    pub fn new(child: W) -> Self {
        Self {
            size: (child.width(), child.height()),
            child,
            coords: Coords::new(0., 0.),
            anchor: (Alignment::Center, Alignment::Center),
            constraint: Constraint::Fixed,
        }
    }
    pub fn coords(&self) -> Coords {
        self.coords
    }
    pub fn constraint(mut self, constraint: Constraint) -> Self {
        self.constraint = constraint;
        self
    }
    pub fn set_constraint(&mut self, constraint: Constraint) {
        self.constraint = constraint;
    }
    pub fn anchor(mut self, x: Alignment, y: Alignment) -> Self {
        self.anchor = (x, y);
        self
    }
    pub fn set_anchor(&mut self, x: Alignment, y: Alignment) {
        self.anchor = (x, y);
    }
}

impl<W: Widget> Deref for WidgetBox<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl<W: Widget> DerefMut for WidgetBox<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}
