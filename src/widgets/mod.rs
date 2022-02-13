//!	This module provides the building blocks of your GUI application.

pub mod button;
pub mod extra;
pub mod image;
pub mod label;
pub mod layout;
pub mod scroll;
pub mod shapes;
pub mod slider;
pub mod window;

use crate::*;
use scroll::Scrollable;
use shapes::Style;
use std::ops::{Deref, DerefMut};
use tiny_skia::*;

pub const START: Alignment = Alignment::Start;
pub const CENTER: Alignment = Alignment::Center;
pub const END: Alignment = Alignment::End;

pub fn blend(pix_a: &Color, pix_b: &Color) -> Color {
    let (r_a, g_a, b_a, a_a) = (pix_a.red(), pix_a.green(), pix_a.blue(), pix_a.alpha());
    let (r_b, g_b, b_b, a_b) = (pix_b.red(), pix_b.green(), pix_b.blue(), pix_b.alpha());
    let red = blend_f32(r_a, r_b, a_b);
    let green = blend_f32(g_a, g_b, a_b);
    let blue = blend_f32(b_a, b_b, a_b);

    Color::from_rgba(red, green, blue, a_b.max(a_a)).unwrap()
}

fn blend_f32(texture: f32, foreground: f32, alpha_fg: f32) -> f32 {
    foreground * alpha_fg + texture * (1. - alpha_fg)
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Alignment {
    Start,
    Center,
    End,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Constraint {
    /// Size remains the same regardless
    Fixed,
    /// The size is the maximum size the widget can take
    Upward,
    /// The size is the minimum size the widget can take
    Downward,
}

impl Geometry for () {
    fn height(&self) -> f32 {
        0.
    }
    fn width(&self) -> f32 {
        0.
    }
}

impl<D> Widget<D> for () {
    fn create_node(&mut self, _: Transform) -> RenderNode {
        RenderNode::None
    }
    fn sync<'d>(&'d mut self, _: &mut SyncContext<D>, _event: Event) -> Damage {
        Damage::None
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, _constraints: &BoxConstraints) -> Size {
        (0., 0.).into()
    }
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

impl<D> Widget<D> for Spacer {
    fn create_node(&mut self, _: Transform) -> RenderNode {
        RenderNode::None
    }
    fn sync<'d>(&'d mut self, _: &mut SyncContext<D>, _event: Event) -> Damage {
        Damage::None
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, _constraints: &BoxConstraints) -> Size {
        (self.width, self.height).into()
    }
}

impl Spacer {
    pub fn width<W: Into<f32>>(width: W) -> Self {
        Self {
            width: width.into(),
            height: 0.,
        }
    }
    pub fn height<H: Into<f32>>(height: H) -> Self {
        Self {
            height: height.into(),
            width: 0.,
        }
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

#[derive(Clone, Debug, PartialEq)]
pub struct Padding<W> {
    pub padding: (f32, f32, f32, f32),
    pub widget: W,
}

impl<W: Geometry> Geometry for Padding<W> {
    fn width(&self) -> f32 {
        let (_, right, _, left) = self.padding;
        self.widget.width() + right + left
    }
    fn height(&self) -> f32 {
        let (top, _, bottom, _) = self.padding;
        self.widget.height() + top + bottom
    }
    fn set_width(&mut self, width: f32) {
        let (_, right, _, left) = self.padding;
        self.widget.set_width(width - right - left)
    }
    fn set_height(&mut self, height: f32) {
        let (top, _, bottom, _) = self.padding;
        self.widget.set_height(height - top - bottom)
    }
    fn maximum_height(&self) -> f32 {
        let (top, _, bottom, _) = self.padding;
        self.widget.maximum_height() + top + bottom
    }
    fn minimum_height(&self) -> f32 {
        let (top, _, bottom, _) = self.padding;
        self.widget.minimum_height() + top + bottom
    }
    fn maximum_width(&self) -> f32 {
        let (_, right, _, left) = self.padding;
        self.widget.maximum_width() + right + left
    }
    fn minimum_width(&self) -> f32 {
        let (_, right, _, left) = self.padding;
        self.widget.minimum_width() + right + left
    }
}

impl<D, W: Widget<D>> Widget<D> for Padding<W> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let (top, _, _, left) = self.padding;
        self.widget.create_node(transform.pre_translate(left, top))
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event) -> Damage {
        let (top, _, _, left) = self.padding;
        if let Event::Pointer(mut x, mut y, p) = event {
            x -= left;
            y -= top;
            self.widget.sync(ctx, Event::Pointer(x, y, p))
        } else {
            self.widget.sync(ctx, event)
        }
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        let (top, right, bottom, left) = self.padding;
        let (width, height) = self
            .widget
            .layout(ctx, &constraints.crop(left + right, top + bottom))
            .into();
        (width + left + right, height + top + bottom).into()
    }
}

impl<W: Scrollable> Scrollable for Padding<W> {
    fn forward(&mut self, step: Option<f32>) {
        self.widget.forward(step)
    }
    fn backward(&mut self, step: Option<f32>) {
        self.widget.backward(step)
    }
    fn inner_height(&self) -> f32 {
        self.widget.inner_height()
    }
    fn inner_width(&self) -> f32 {
        self.widget.inner_width()
    }
    fn orientation(&self) -> Orientation {
        self.widget.orientation()
    }
    fn position(&self) -> f32 {
        self.widget.position()
    }
}

impl<W> Padding<W> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            padding: (0., 0., 0., 0.),
        }
    }
    pub fn set_padding_top(&mut self, padding: f32) {
        self.padding.0 = padding;
    }
    pub fn set_padding_right(&mut self, padding: f32) {
        self.padding.1 = padding;
    }
    pub fn set_padding_bottom(&mut self, padding: f32) {
        self.padding.2 = padding;
    }
    pub fn set_padding_left(&mut self, padding: f32) {
        self.padding.3 = padding;
    }
    pub fn padding_top(mut self, padding: f32) -> Self {
        self.set_padding_top(padding);
        self
    }
    pub fn padding_right(mut self, padding: f32) -> Self {
        self.set_padding_right(padding);
        self
    }
    pub fn padding_bottom(mut self, padding: f32) -> Self {
        self.set_padding_bottom(padding);
        self
    }
    pub fn padding_left(mut self, padding: f32) -> Self {
        self.set_padding_left(padding);
        self
    }
    pub fn set_padding(&mut self, padding: f32) {
        self.set_padding_top(padding);
        self.set_padding_right(padding);
        self.set_padding_bottom(padding);
        self.set_padding_left(padding);
    }
    pub fn padding(mut self, padding: f32) -> Self {
        self.set_padding(padding);
        self
    }
}

impl<W> Deref for Padding<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<W> DerefMut for Padding<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}

use layout::Positioner;

/// Applies constraint to a widget's dimension.
#[derive(Clone, Debug, PartialEq)]
pub struct WidgetBox<W> {
    pub(crate) widget: Positioner<W>,
    width: Option<f32>,
    height: Option<f32>,
    size: Size,
    constraint: Constraint,
    anchor: (Alignment, Alignment),
}

impl<W: Geometry> Geometry for WidgetBox<W> {
    fn width(&self) -> f32 {
        self.size.width
    }
    fn height(&self) -> f32 {
        self.size.height
    }
    fn maximum_height(&self) -> f32 {
        match self.height {
            Some(l_height) => match self.constraint {
                Constraint::Upward | Constraint::Fixed => l_height,
                Constraint::Downward => std::f32::INFINITY,
            },
            None => std::f32::INFINITY,
        }
    }
    fn maximum_width(&self) -> f32 {
        match self.width {
            Some(l_width) => match self.constraint {
                Constraint::Upward | Constraint::Fixed => l_width,
                Constraint::Downward => std::f32::INFINITY,
            },
            None => std::f32::INFINITY,
        }
    }
}

impl<W> WidgetBox<W> {
    fn minimum_width_from(&self, width: f32) -> f32 {
        match self.width {
            Some(l_width) => match self.constraint {
                Constraint::Fixed => l_width,
                Constraint::Upward => width.min(l_width),
                Constraint::Downward => width.max(l_width),
            },
            None => width,
        }
    }
    fn minimum_height_from(&self, height: f32) -> f32 {
        match self.height {
            Some(l_height) => match self.constraint {
                Constraint::Fixed => l_height,
                Constraint::Upward => height.min(l_height),
                Constraint::Downward => height.max(l_height),
            },
            None => height,
        }
    }
}

impl<W> GeometryExt for WidgetBox<W> {
    fn apply_width(&mut self, width: f32) {
        self.width = Some(width);
    }
    fn apply_height(&mut self, height: f32) {
        self.height = Some(height);
    }
}

impl<D, W: Widget<D>> Widget<D> for WidgetBox<W> {
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event) -> Damage {
        self.widget.sync(ctx, event)
    }
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        self.widget.create_node(transform)
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        let mut width = match self.width {
            Some(l_width) => match self.constraint {
                Constraint::Fixed => l_width,
                Constraint::Upward => l_width.min(constraints.maximum_width()),
                Constraint::Downward => l_width.max(constraints.minimum_width()),
            },
            None => constraints.minimum_width(),
        };
        let mut height = match self.height {
            Some(l_height) => match self.constraint {
                Constraint::Fixed => l_height,
                Constraint::Upward => l_height.min(constraints.maximum_height()),
                Constraint::Downward => l_height.max(constraints.minimum_height()),
            },
            None => constraints.minimum_height(),
        };
        let (inner_width, inner_height) = self
            .widget
            .layout(
                ctx,
                &match self.constraint {
                    Constraint::Fixed => BoxConstraints::new((width, height), (width, height)),
                    _ => constraints
                        .with_min(
                            self.minimum_width_from(constraints.minimum_width()),
                            self.minimum_height_from(constraints.minimum_height()),
                        )
                        .with_max(
                            self.maximum_width().min(constraints.maximum_width()),
                            self.maximum_height().min(constraints.maximum_height()),
                        ),
                },
            )
            .into();
        width = width.max(inner_width);
        height = height.max(inner_height);
        let (horizontal, vertical) = &self.anchor;
        let dx = match horizontal {
            Alignment::Start => 0.,
            Alignment::Center => ((width - inner_width) / 2.).floor(),
            Alignment::End => (width - inner_width).floor(),
        };
        let dy = match vertical {
            Alignment::Start => 0.,
            Alignment::Center => ((height - inner_height) / 2.).floor(),
            Alignment::End => (height - inner_height).floor(),
        };
        self.size = (width, height).into();
        self.widget.set_coords(dx, dy);
        self.size.into()
    }
}

impl<W: Geometry> WidgetBox<W> {
    pub fn new(widget: W) -> Self {
        Self {
            size: Size::new(0., 0.),
            widget: Positioner::new(widget),
            width: None,
            height: None,
            anchor: (Alignment::Center, Alignment::Center),
            constraint: Constraint::Downward,
        }
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

impl<W> Deref for WidgetBox<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<W> DerefMut for WidgetBox<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}
