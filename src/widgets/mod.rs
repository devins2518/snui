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
pub use container::*;
pub use shapes::Style;
use std::ops::{Deref, DerefMut};
use tiny_skia::*;

pub const START: Alignment = Alignment::Start;
pub const CENTER: Alignment = Alignment::Center;
pub const END: Alignment = Alignment::End;

pub fn blend(pix_a: &Color, pix_b: &Color) -> Color {
    let (r_a, g_a, b_a) = (pix_a.red(), pix_a.green(), pix_a.blue());
    let (r_b, g_b, b_b, a_b) = (pix_b.red(), pix_b.green(), pix_b.blue(), pix_b.alpha());
    let red = blend_f32(r_a, r_b, a_b);
    let green = blend_f32(g_a, g_b, a_b);
    let blue = blend_f32(b_a, b_b, a_b);

    Color::from_rgba(red, green, blue, 1.).unwrap()
}

fn blend_f32(background: f32, foreground: f32, alpha_fg: f32) -> f32 {
    foreground * alpha_fg + background * (1. - alpha_fg)
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

// Simple dump widget with a fixed size.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Spacer {
    pub width: f32,
    pub height: f32,
}

impl Geometry for Spacer {
    fn height(&self) -> f32 {
        self.height
    }
    fn width(&self) -> f32 {
        self.width
    }
}

impl Widget for Spacer {
    fn create_node(&mut self, _x: f32, _y: f32) -> RenderNode {
        RenderNode::None
    }
    fn sync<'d>(&'d mut self, _ctx: &mut SyncContext, _event: Event) -> Damage {
        Damage::None
    }
}

impl Default for Spacer {
    fn default() -> Self {
        Self {
            width: 0.,
            height: 0.,
        }
    }
}

pub struct Padding<W: Widget> {
    pub padding: (f32, f32, f32, f32),
    pub widget: W,
}

impl<W: Widget> Geometry for Padding<W> {
    fn width(&self) -> f32 {
        let (_, right, _, left) = self.padding;
        self.widget.width() + right + left
    }
    fn height(&self) -> f32 {
        let (top, _, bottom, _) = self.padding;
        self.widget.height() + top + bottom
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        let (_, right, _, left) = self.padding;
        self.widget.set_width(width - right - left)?;
        Ok(())
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        let (top, _, bottom, _) = self.padding;
        self.widget.set_height(height - top - bottom)?;
        Ok(())
    }
}

impl<W: Widget> Widget for Padding<W> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let (top, _, _, left) = self.padding;
        self.widget.create_node(x + left, y + top)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) -> Damage {
        let (top, _, _, left) = self.padding;
        if let Event::Pointer(mut x, mut y, p) = event {
            x -= left;
            y -= top;
            self.widget.sync(ctx, Event::Pointer(x, y, p))
        } else {
            self.widget.sync(ctx, event)
        }
    }
}

impl<W: Widget> Padding<W> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            padding: (0., 0., 0., 0.),
        }
    }
    pub fn set_padding(&mut self, top: f32, right: f32, bottom: f32, left: f32) {
        self.padding = (top, right, bottom, left);
    }
    pub fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.set_padding(top, right, bottom, left);
        self
    }
    pub fn even_padding(self, padding: f32) -> Self {
        self.padding(padding, padding, padding, padding)
    }
    pub fn set_even_padding(&mut self, padding: f32) {
        self.set_padding(padding, padding, padding, padding);
    }
}

impl<W: Widget> Deref for Padding<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<W: Widget> DerefMut for Padding<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}

pub struct WidgetBox<W: Widget> {
    widget: W,
    coords: Coords,
    size: (f32, f32),
    constraint: Constraint,
    anchor: (Alignment, Alignment),
}

impl<W: Widget> Geometry for WidgetBox<W> {
    fn width(&self) -> f32 {
        match &self.constraint {
            Constraint::Fixed => self.size.0,
            Constraint::Upward => self.widget.width().min(self.size.0),
            Constraint::Downward => self.widget.width().max(self.size.0),
        }
    }
    fn height(&self) -> f32 {
        match &self.constraint {
            Constraint::Fixed => self.size.1,
            Constraint::Upward => self.widget.height().min(self.size.1),
            Constraint::Downward => self.widget.height().max(self.size.1),
        }
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        if width > 0. {
            self.size.0 = width;
            return Ok(());
        }
        Err(self.size.0)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        if height > 0. {
            self.size.1 = height;
            return Ok(());
        }
        Err(self.size.1)
    }
}

impl<W: Widget> Widget for WidgetBox<W> {
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) -> Damage {
        if let Event::Pointer(mut x, mut y, pointer) = event {
            x -= self.coords.x;
            y -= self.coords.y;
            self.widget.sync(ctx, Event::Pointer(x, y, pointer))
        } else {
            self.widget.sync(ctx, event)
        }
    }
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        if (self.constraint == Constraint::Fixed || self.constraint == Constraint::Upward)
            && (self.widget.width() > self.size.0 || self.widget.height() > self.size.1)
        {
            eprintln!(
                "Position: {} x {}\nWidgetBox exceeded bounds: {} x {}",
                x,
                y,
                self.width(),
                self.height()
            );
            return RenderNode::None;
        }
        let (horizontal, vertical) = &self.anchor;
        let dx = match horizontal {
            Alignment::Start => 0.,
            Alignment::Center => ((self.width() - self.widget.width()) / 2.).floor(),
            Alignment::End => (self.width() - self.widget.width()).floor(),
        };
        let dy = match vertical {
            Alignment::Start => 0.,
            Alignment::Center => ((self.height() - self.widget.height()) / 2.).floor(),
            Alignment::End => (self.height() - self.widget.height()).floor(),
        };
        self.coords = Coords::new(dx, dy);
        let node = self.widget.create_node(x + dx, y + dy);
        if node.is_none() {
            return node;
        }
        RenderNode::Extension {
            background: scene::Instruction::new(
                x,
                y,
                shapes::Rectangle::empty(self.width(), self.height()),
            ),
            border: None,
            node: Box::new(node),
        }
    }
}

impl<W: Widget> WidgetBox<W> {
    pub fn new(widget: W) -> Self {
        Self {
            size: (widget.width(), widget.height()),
            widget,
            coords: Coords::new(0., 0.),
            anchor: (Alignment::Center, Alignment::Center),
            constraint: Constraint::Downward,
        }
    }
    pub fn size(mut self, width: f32, height: f32) -> Self {
        let _ = self.set_size(width, height);
        self.size = (width, height);
        self
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
        &self.widget
    }
}

impl<W: Widget> DerefMut for WidgetBox<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}
