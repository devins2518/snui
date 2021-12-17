pub mod rectangle;

use crate::scene::*;
use crate::*;
pub use rectangle::Rectangle;
use std::f32::consts::FRAC_1_SQRT_2;
use std::ops::{Deref, DerefMut};

pub trait Style: Sized {
    fn radius(self, tl: f32, tr: f32, br: f32, bl: f32) -> Self;
    fn even_radius(self, radius: f32) -> Self {
        self.radius(radius, radius, radius, radius)
    }
    fn background<B: Into<Background>>(self, background: B) -> Self;
    fn border_width(self, width: f32) -> Self;
    fn border_color(self, color: u32) -> Self;
    fn border(self, color: u32, width: f32) -> Self;
    fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32);
    fn set_even_radius(&mut self, radius: f32) {
        self.set_radius(radius, radius, radius, radius);
    }
    fn set_background<B: Into<Background>>(&mut self, background: B);
    fn set_border_width(&mut self, width: f32);
    fn set_border_color(&mut self, color: u32);
    fn set_border(&mut self, color: u32, width: f32);
}

#[derive(Clone, PartialEq, Debug)]
pub enum ShapeStyle {
    Background(Background),
    Border(Color, f32),
}

impl From<u32> for ShapeStyle {
    fn from(color: u32) -> Self {
        ShapeStyle::Background(color.into())
    }
}

pub struct WidgetExt<W: Widget> {
    child: W,
    padding: [f32; 4],
    border: Rectangle,
    background: Rectangle,
}

impl<W: Widget> WidgetExt<W> {
    pub fn new(child: W) -> Self {
        let width = child.width();
        let height = child.height();
        WidgetExt {
            child,
            border: Rectangle::empty(width, height),
            background: Rectangle::empty(width, height),
            padding: [0.; 4],
        }
    }
    fn inner_width(&self) -> f32 {
        self.child.width() + self.padding[1] + self.padding[3]
    }
    fn inner_height(&self) -> f32 {
        self.child.height() + self.padding[0] + self.padding[2]
    }
    // The padding will be rounded automatically
    pub fn force_padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.padding = [top.round(), right.round(), bottom.round(), left.round()];
        self
    }
    pub fn even_padding(self, padding: f32) -> Self {
        self.padding(padding, padding, padding, padding)
    }
    // The padding will be rounded automatically
    pub fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        let (tl, tr, br, bl) = self.background.radius;
        let min = minimum_padding(tl, tr, br, bl);
        self.padding = [
            top.round().max(min),
            right.round().max(min),
            bottom.round().max(min),
            left.round().max(min),
        ];
        self
    }
    pub fn set_even_padding(&mut self, padding: f32) {
        self.set_padding(padding, padding, padding, padding);
    }
    pub fn set_padding(&mut self, top: f32, right: f32, bottom: f32, left: f32) {
        let (tl, tr, br, bl) = self.background.radius;
        let min = minimum_padding(tl, tr, br, bl);
        self.padding = [
            top.round().max(min),
            right.round().max(min),
            bottom.round().max(min),
            left.round().max(min),
        ];
    }
    pub fn unwrap(self) -> W {
        self.child
    }
}

fn minimum_padding(tl: f32, tr: f32, br: f32, bl: f32) -> f32 {
    let max = tl.max(tr).max(br).max(bl);
    let radius = max - (max * FRAC_1_SQRT_2);
    return radius.ceil();
}

impl<W: Widget> Geometry for WidgetExt<W> {
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        let border = if let ShapeStyle::Border(_, size) = self.border.get_style() {
            *size
        } else {
            0.
        };
        self.background.set_width(width - border)?;
        self.border.set_width(width - border)?;
        self.child
            .set_width(width - self.padding[1] - self.padding[3] - border)?;
        Ok(())
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        let border = if let ShapeStyle::Border(_, size) = self.border.get_style() {
            *size
        } else {
            0.
        };
        self.background.set_height(height - border)?;
        self.border.set_height(height - border)?;
        self.child
            .set_height(height - self.padding[0] - self.padding[2] - border)?;
        Ok(())
    }
    fn width(&self) -> f32 {
        if let ShapeStyle::Border(_, size) = &self.border.style {
            return self.inner_width() + size;
        }
        self.inner_width()
    }
    fn height(&self) -> f32 {
        if let ShapeStyle::Border(_, size) = &self.border.style {
            return self.inner_height() + size;
        }
        self.inner_height()
    }
}

impl<W: Widget + Style> WidgetExt<W> {
    pub fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.child.set_radius(tl, tr, br, bl);
        let delta = minimum_padding(tl, tr, br, bl);
        Style::set_radius(self, tl + delta, tr + delta, br + delta, bl + delta)
    }
    pub fn radius(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        self.child.set_radius(tl, tr, br, bl);
        let delta = minimum_padding(tl, tr, br, bl);
        Style::radius(self, tl + delta, tr + delta, br + delta, bl + delta)
    }
    pub fn even_radius(self, radius: f32) -> Self {
        WidgetExt::radius(self, radius, radius, radius, radius)
    }
    pub fn set_even_radius(&mut self, radius: f32) {
        WidgetExt::set_radius(self, radius, radius, radius, radius);
    }
}

impl<W: Widget> Style for WidgetExt<W> {
    fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.background.set_radius(tl, tr, br, bl);
        self.border.set_radius(tl, tr, br, bl);
        self.set_padding(
            self.padding[0],
            self.padding[1],
            self.padding[2],
            self.padding[3],
        );
    }
    fn radius(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        let width = self.inner_width();
        let height = self.inner_height();
        let background = {
            self.background.set_size(width, height).unwrap();
            self.background.radius(tl, tr, br, bl)
        };
        let border = {
            self.border.set_size(width, height).unwrap();
            self.border.radius(tl, tr, br, bl)
        };
        Self {
            border,
            background,
            child: self.child,
            padding: self.padding,
        }
        .padding(
            self.padding[0],
            self.padding[1],
            self.padding[2],
            self.padding[3],
        )
    }
    fn set_background<B: Into<Background>>(&mut self, background: B) {
        self.background.set_background(background.into());
    }
    fn background<B: Into<Background>>(mut self, background: B) -> Self {
        let width = self.inner_width();
        let height = self.inner_height();
        let _ = self.background.set_size(width, height);
        let _ = self.border.set_size(width, height);
        Self {
            background: self.background.background(background),
            border: self.border,
            child: self.child,
            padding: self.padding,
        }
    }
    fn set_border(&mut self, color: u32, width: f32) {
        self.border.set_border(color, width);
    }
    fn border(mut self, color: u32, size: f32) -> Self {
        let width = self.inner_width();
        let height = self.inner_height();
        let _ = self.background.set_size(width, height);
        let _ = self.border.set_size(width, height);
        Self {
            border: self.border.border(color, size),
            background: self.background,
            child: self.child,
            padding: self.padding,
        }
    }
    fn set_border_width(&mut self, width: f32) {
        self.border.set_border_width(width);
    }
    fn border_width(mut self, size: f32) -> Self {
        let width = self.inner_width();
        let height = self.inner_height();
        let _ = self.background.set_size(width, height);
        let _ = self.border.set_size(width, height);
        Self {
            border: self.border.border_width(size),
            background: self.background,
            child: self.child,
            padding: self.padding,
        }
    }
    fn set_border_color(&mut self, color: u32) {
        self.border.set_border_color(color);
    }
    fn border_color(mut self, color: u32) -> Self {
        let width = self.inner_width();
        let height = self.inner_height();
        let _ = self.background.set_size(width, height);
        let _ = self.border.set_size(width, height);
        Self {
            border: self.border.border_color(color),
            background: self.background,
            child: self.child,
            padding: self.padding,
        }
    }
}

impl<W: Widget> Widget for WidgetExt<W> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let border = if let ShapeStyle::Border(_, size) = self.border.get_style() {
            (*size / 2.).round()
        } else {
            0.
        };
        RenderNode::Extension {
            node: Box::new(
                self.child
                    .create_node(x + self.padding[3] + border, y + self.padding[0] + border),
            ),
            border: Some({
                let width = self.inner_width();
                let height = self.inner_height();
                self.border.set_size(width, height).unwrap();
                if let RenderNode::Instruction(rect) = self.border.create_node(x, y) {
                    rect
                } else {
                    unreachable!()
                }
            }),
            background: {
                let width = self.inner_width();
                let height = self.inner_height();
                self.background.set_size(width, height).unwrap();
                if let RenderNode::Instruction(rect) =
                    self.background.create_node(x + border, y + border)
                {
                    rect
                } else {
                    unreachable!()
                }
            },
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) -> Damage {
        if let Event::Pointer(mut x, mut y, p) = event {
            let border = if let ShapeStyle::Border(_, size) = self.border.get_style() {
                *size
            } else {
                0.
            };
            x -= border + self.padding[3];
            y -= border + self.padding[0];
            self.child.sync(ctx, Event::Pointer(x, y, p))
        } else {
            self.child.sync(ctx, event)
        }
    }
}

impl<W: Widget> Deref for WidgetExt<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl<W: Widget> DerefMut for WidgetExt<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}
