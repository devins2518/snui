use crate::*;
use crate::scene::*;
use crate::widgets::shapes::Style;
pub use crate::widgets::shapes::rectangle::Rectangle;
use std::f32::consts::FRAC_1_SQRT_2;
use std::ops::{Deref, DerefMut};
use widgets::Padding;

/// Main type used for styling.
/// Any widget can be wrapped in a WidgetStyle and take advantage of the Style trait.
pub struct WidgetStyle<W> {
    widget: Padding<W>,
    radius: (f32, f32, f32, f32),
    background: Texture,
    border: (Texture, f32),
}

impl<W: Geometry> WidgetStyle<W> {
    pub fn new(widget: W) -> Self {
        WidgetStyle {
            widget: Padding::new(widget),
            background: Texture::Transparent,
            border: (Texture::Transparent, 0.),
            radius: (0., 0., 0., 0.),
        }
    }
    fn inner_width(&self) -> f32 {
        self.widget.width()
    }
    fn inner_height(&self) -> f32 {
        self.widget.height()
    }
    pub fn set_padding(&mut self, top: f32, right: f32, bottom: f32, left: f32) {
        self.widget.padding = (top, right, bottom, left);
    }
    pub fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.widget.set_padding(top, right, bottom, left);
        self
    }
    pub fn even_padding(mut self, padding: f32) -> Self {
        self.widget.set_padding(padding, padding, padding, padding);
        self
    }
    pub fn set_even_padding(&mut self, padding: f32) {
        self.widget.set_padding(padding, padding, padding, padding);
    }
}

fn minimum_padding(tl: f32, tr: f32, br: f32, bl: f32) -> f32 {
    let max = tl.max(tr).max(br).max(bl);
    let radius = max * FRAC_1_SQRT_2;
    return radius.floor();
}

impl<W: Geometry> Geometry for WidgetStyle<W> {
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        let border = self.border.1;
        if let Err(width) = self.widget.set_width(width - 2. * border) {
            Err(width + 2. * border)
        } else {
            Ok(())
        }
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        let border = self.border.1;
        if let Err(height) = self.widget.set_height(height - 2. * border) {
            Err(height + 2. * border)
        } else {
            Ok(())
        }
    }
    fn width(&self) -> f32 {
        self.inner_width() + 2. * self.border.1
    }
    fn height(&self) -> f32 {
        self.inner_height() + 2. * self.border.1
    }
}

impl<W: Style> WidgetStyle<W> {
    pub fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.widget.set_radius(tl, tr, br, bl);
        let delta = minimum_padding(tl, tr, br, bl);
        self.widget.padding.0 = self.widget.padding.0.max(delta);
        self.widget.padding.1 = self.widget.padding.1.max(delta);
        self.widget.padding.2 = self.widget.padding.2.max(delta);
        self.widget.padding.3 = self.widget.padding.3.max(delta);
        self.radius = (tl + delta, tr + delta, br + delta, bl + delta);
    }
    pub fn radius(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        self.widget.set_radius(tl, tr, br, bl);
        WidgetStyle::set_radius(&mut self, tl, tr, br, bl);
        self
    }
    pub fn even_radius(self, radius: f32) -> Self {
        WidgetStyle::radius(self, radius, radius, radius, radius)
    }
    pub fn set_even_radius(&mut self, radius: f32) {
        WidgetStyle::set_radius(self, radius, radius, radius, radius);
    }
}

impl<W> Style for WidgetStyle<W> {
    fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        let delta = minimum_padding(tl, tr, br, bl);
        self.widget.padding.0 = self.widget.padding.0.max(delta);
        self.widget.padding.1 = self.widget.padding.1.max(delta);
        self.widget.padding.2 = self.widget.padding.2.max(delta);
        self.widget.padding.3 = self.widget.padding.3.max(delta);
        self.radius = (tl, tr, br, bl);
    }
    fn set_background<B: Into<Texture>>(&mut self, texture: B) {
        self.background = texture.into();
    }
    fn set_border_size(&mut self, size: f32) {
        self.border.1 = size;
    }
    fn set_border_texture<T: Into<Texture>>(&mut self, texture: T) {
        self.border.0 = texture.into();
    }
}

impl<D, W: Widget<D>> Widget<D> for WidgetStyle<W> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let x = transform.tx;
        let y = transform.ty;
        let (border_texture, border_size) = self.border.clone();
        let node = self
            .widget
            .create_node(transform.pre_translate(border_size, border_size));
        let width = self.inner_width();
        let height = self.inner_height();
        match &mut self.background {
            Texture::Image(coords, _) => {
                coords.x = x + border_size;
                coords.y = y + border_size;
            }
            Texture::LinearGradient {
                start,
                end,
                angle,
                stops: _,
                mode: _,
            } => {
                start.x = x + border_size;
                start.y = y + border_size;
                end.x = start.x + width;
                end.y = start.y + height * angle.tan();
            }
            _ => {}
        }
        if node.is_none() {
            return RenderNode::None;
        }
        RenderNode::Extension {
            node: Box::new(node),
            border: {
                if border_texture != Texture::Transparent || border_size > 0. {
                    Some(Instruction::new(
                        transform,
                        Rectangle::empty(width, height)
                            .radius(self.radius.0, self.radius.1, self.radius.2, self.radius.3)
                            .border(border_texture, border_size),
                    ))
                } else {
                    None
                }
            },
            background: Instruction::new(
                transform.pre_translate(border_size, border_size),
                Rectangle::empty(width, height)
                    .background(self.background.clone())
                    .radius(
                        minimum_radius(self.radius.0, border_size),
                        minimum_radius(self.radius.1, border_size),
                        minimum_radius(self.radius.2, border_size),
                        minimum_radius(self.radius.3, border_size),
                    ),
            ),
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event) -> Damage {
        if let Event::Pointer(mut x, mut y, p) = event {
            let border = self.border.1;
            x -= border;
            y -= border;
            self.widget.sync(ctx, Event::Pointer(x, y, p))
        } else {
            self.widget.sync(ctx, event)
        }
    }
}

fn minimum_radius(radius: f32, border: f32) -> f32 {
    if border > radius {
        return 0.;
    }
    radius - (border / 2.)
}

impl<W> Deref for WidgetStyle<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        self.widget.deref()
    }
}

impl<W> DerefMut for WidgetStyle<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.widget.deref_mut()
    }
}
