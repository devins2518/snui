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
    padding: (f32, f32, f32, f32),
    radius: (f32, f32, f32, f32),
    background: Background,
    border: (f32, u32),
}

impl<W: Widget> WidgetExt<W> {
    pub fn new(child: W) -> Self {
        WidgetExt {
            child,
            background: Background::Transparent,
            border: (0., 0),
            radius: (0., 0., 0., 0.),
            padding: (0., 0., 0., 0.),
        }
    }
    fn inner_width(&self) -> f32 {
        let (_, right, _, left) = self.padding;
        self.child.width() + right + left
    }
    fn inner_height(&self) -> f32 {
        let (top, _, bottom, _) = self.padding;
        self.child.height() + top + bottom
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

fn minimum_padding(tl: f32, tr: f32, br: f32, bl: f32) -> f32 {
    let max = tl.max(tr).max(br).max(bl);
    let radius = max * FRAC_1_SQRT_2;
    return radius.floor();
}

impl<W: Widget> Geometry for WidgetExt<W> {
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        let border = self.border.0;
        let (_, right, _, left) = self.padding;
        self.child
            .set_width(width - right - left - border)?;
        Ok(())
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        let border = self.border.0;
        let (top, _, bottom, _) = self.padding;
        self.child
            .set_height(height - top - bottom - border)?;
        Ok(())
    }
    fn width(&self) -> f32 {
        self.inner_width() + self.border.0
    }
    fn height(&self) -> f32 {
        self.inner_height() + self.border.0
    }
}

impl<W: Widget + Style> WidgetExt<W> {
    pub fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.child.set_radius(tl, tr, br, bl);
        let delta = minimum_padding(tl, tr, br, bl);
        self.padding.0 = self.padding.0.max(delta);
        self.padding.1 = self.padding.1.max(delta);
        self.padding.2 = self.padding.2.max(delta);
        self.padding.3 = self.padding.3.max(delta);
        self.radius = (tl + delta, tr + delta, br + delta, bl + delta);
    }
    pub fn radius(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        self.child.set_radius(tl, tr, br, bl);
        WidgetExt::set_radius(&mut self, tl, tr, br, bl);
        self
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
        let delta = minimum_padding(tl, tr, br, bl);
        self.padding.0 = self.padding.0.max(delta);
        self.padding.1 = self.padding.1.max(delta);
        self.padding.2 = self.padding.2.max(delta);
        self.padding.3 = self.padding.3.max(delta);
        self.radius = (tl, tr, br, bl);
    }
    fn radius(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        self.set_radius(tl, tr, br, bl);
        self
    }
    fn set_background<B: Into<Background>>(&mut self, background: B) {
        self.background = background.into();
    }
    fn background<B: Into<Background>>(mut self, background: B) -> Self {
        self.set_background(background);
        self
    }
    fn set_border(&mut self, color: u32, width: f32) {
        self.border = (width, color);
    }
    fn border(mut self, color: u32, size: f32) -> Self {
        self.set_border(color, size);
        self
    }
    fn set_border_width(&mut self, width: f32) {
        self.border.0 = width;
    }
    fn border_width(mut self, size: f32) -> Self {
        self.set_border_width(size);
        self
    }
    fn set_border_color(&mut self, color: u32) {
        self.border.1 = color;
    }
    fn border_color(mut self, color: u32) -> Self {
        self.set_border_color(color);
        self
    }
}

impl<W: Widget> Widget for WidgetExt<W> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let (border_size, border_color) = self.border;
        let node = self.child.create_node(
            x + self.padding.3 + border_size,
            y + self.padding.0 + border_size,
        );
        RenderNode::Extension {
            node: Box::new(node),
            border: {
                if border_color != 0 || border_size > 0. {
                    Some(Instruction::new(
                        x,
                        y,
                        Rectangle::empty(self.inner_width(), self.inner_height())
                            .radius(self.radius.0, self.radius.1, self.radius.2, self.radius.3)
                            .border(border_color, border_size),
                    ))
                } else {
                    None
                }
            },
            background: Instruction::new(
                x + border_size,
                y + border_size,
                Rectangle::empty(self.inner_width(), self.inner_height())
                    .radius(self.radius.0, self.radius.1, self.radius.2, self.radius.3)
                    .background(self.background.clone()),
            ),
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) -> Damage {
        if let Event::Pointer(mut x, mut y, p) = event {
            let border = self.border.0;
            x -= border + self.padding.3;
            y -= border + self.padding.0;
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
