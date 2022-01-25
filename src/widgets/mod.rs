pub mod button;
pub mod container;
pub mod extra;
pub mod image;
pub mod shapes;
pub mod slider;
pub mod text;
pub mod window;

use crate::scene::Coords;
pub use crate::widgets::image::InnerImage;
use crate::*;
pub use button::Button;
pub use container::*;
pub use shapes::Style;
pub use slider::Slider;
use std::marker::PhantomData;
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

fn blend_f32(texture: f32, foreground: f32, alpha_fg: f32) -> f32 {
    foreground * alpha_fg + texture * (1. - alpha_fg)
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

impl Geometry for () {
    fn height(&self) -> f32 {
        0.
    }
    fn width(&self) -> f32 {
        0.
    }
}

impl<M> Widget<M> for () {
    fn create_node(&mut self, _: Transform) -> RenderNode {
        RenderNode::None
    }
    fn sync<'d>(&'d mut self, _ctx: &mut SyncContext<M>, _event: Event<M>) -> Damage {
        Damage::None
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

impl<M> Widget<M> for Spacer {
    fn create_node(&mut self, _: Transform) -> RenderNode {
        RenderNode::None
    }
    fn sync<'d>(&'d mut self, _ctx: &mut SyncContext<M>, _event: Event<M>) -> Damage {
        Damage::None
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

pub struct Padding<M, W: Widget<M>> {
    pub padding: (f32, f32, f32, f32),
    pub widget: W,
    _request: PhantomData<M>,
}

impl<M, W: Widget<M>> Geometry for Padding<M, W> {
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
        if let Err(width) = self.widget.set_width(width - right - left) {
            Err(width + right + left)
        } else {
            Ok(())
        }
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        let (top, _, bottom, _) = self.padding;
        if let Err(height) = self.widget.set_height(height - top - bottom) {
            Err(height + top + bottom)
        } else {
            Ok(())
        }
    }
}

impl<M, W: Widget<M>> Widget<M> for Padding<M, W> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let (top, _, _, left) = self.padding;
        self.widget.create_node(transform.pre_translate(left, top))
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<M>) -> Damage {
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

impl<M, W: Widget<M>> Padding<M, W> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            padding: (0., 0., 0., 0.),
            _request: PhantomData,
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

impl<M, W: Widget<M>> Deref for Padding<M, W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<M, W: Widget<M>> DerefMut for Padding<M, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}

pub struct WidgetBox<M, W: Widget<M>> {
    pub(crate) widget: W,
    coords: Coords,
    width: Option<f32>,
    height: Option<f32>,
    constraint: Constraint,
    anchor: (Alignment, Alignment),
    _request: PhantomData<M>,
}

impl<M, W: Widget<M>> Geometry for WidgetBox<M, W> {
    fn width(&self) -> f32 {
        match &self.constraint {
            Constraint::Fixed => self.width.unwrap_or(self.widget.width()),
            Constraint::Upward => {
                if let Some(width) = self.width {
                    self.widget.width().min(width)
                } else {
                    self.widget.width()
                }
            }
            Constraint::Downward => self.widget.width().max(self.width.unwrap_or(0.)),
        }
    }
    fn height(&self) -> f32 {
        match &self.constraint {
            Constraint::Fixed => self.height.unwrap_or(self.widget.height()),
            Constraint::Upward => {
                if let Some(height) = self.height {
                    self.widget.height().min(height)
                } else {
                    self.widget.height()
                }
            }
            Constraint::Downward => self.widget.height().max(self.height.unwrap_or(0.)),
        }
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        if width > 0. {
            match &self.constraint {
                Constraint::Fixed | Constraint::Upward => {
                    if width < self.widget.width() {
                        if let Err(width) = self.widget.set_width(width) {
                            self.width = Some(width);
                        }
                    }
                }
                Constraint::Downward => {
                    let ww = self.widget.width();
                    if width < ww {
                        self.width = None;
                        return Err(ww);
                    } else {
                        self.width = Some(width);
                    }
                }
            }
            return Ok(());
        }
        Err(self.width())
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        if height > 0. {
            match &self.constraint {
                Constraint::Fixed | Constraint::Upward => {
                    if height < self.widget.height() {
                        if let Err(height) = self.widget.set_height(height) {
                            self.height = Some(height);
                        }
                    }
                }
                Constraint::Downward => {
                    let wh = self.widget.height();
                    if height < wh {
                        self.height = None;
                        return Err(wh);
                    } else {
                        self.height = Some(height);
                    }
                }
            }
            return Ok(());
        }
        Err(self.height())
    }
}

impl<M, W: Widget<M>> Widget<M> for WidgetBox<M, W> {
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<M>) -> Damage {
        if let Event::Pointer(mut x, mut y, pointer) = event {
            x -= self.coords.x;
            y -= self.coords.y;
            self.widget.sync(ctx, Event::Pointer(x, y, pointer))
        } else {
            self.widget.sync(ctx, event)
        }
    }
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        if (self.constraint == Constraint::Fixed || self.constraint == Constraint::Upward)
            && (self.widget.width() > self.width() || self.widget.height() > self.height())
        {
            eprintln!(
                "Position: {} x {}\nWidgetBox exceeded bounds: {} x {}",
                transform.tx,
                transform.ty,
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
        self.widget.create_node(transform.pre_translate(dx, dy))
    }
}

impl<M, W: Widget<M>> WidgetBox<M, W> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            width: None,
            height: None,
            coords: Coords::new(0., 0.),
            anchor: (Alignment::Center, Alignment::Center),
            constraint: Constraint::Downward,
            _request: PhantomData,
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

impl<M, W: Widget<M>> Deref for WidgetBox<M, W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<M, W: Widget<M>> DerefMut for WidgetBox<M, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}
