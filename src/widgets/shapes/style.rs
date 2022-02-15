use crate::scene::*;
use crate::widgets::shapes::rectangle::Rectangle;
use crate::widgets::shapes::Style;
use crate::*;
use std::f32::consts::FRAC_1_SQRT_2;
use std::ops::{Deref, DerefMut};
use widgets::Padding;

/// Main type used for styling.
/// Any widget can be wrapped in a WidgetStyle and take advantage of the Style trait.
#[derive(Debug)]
pub struct WidgetStyle<W> {
    size: Size,
    widget: Padding<W>,
    radius: (f32, f32, f32, f32),
    background: Texture,
    border: (Texture, f32),
}

impl<W: Geometry> WidgetStyle<W> {
    pub fn new(widget: W) -> Self {
        WidgetStyle {
            size: Size::new(0., 0.),
            widget: Padding::new(widget),
            background: Texture::Transparent,
            border: (Texture::Transparent, 0.),
            radius: (0., 0., 0., 0.),
        }
    }
    pub fn padding_top(mut self, padding: f32) -> Self {
        self.widget.set_padding_top(padding);
        self
    }
    pub fn padding_right(mut self, padding: f32) -> Self {
        self.widget.set_padding_right(padding);
        self
    }
    pub fn padding_bottom(mut self, padding: f32) -> Self {
        self.widget.set_padding_bottom(padding);
        self
    }
    pub fn padding_left(mut self, padding: f32) -> Self {
        self.widget.set_padding_left(padding);
        self
    }
    pub fn set_padding(&mut self, padding: f32) {
        self.widget.set_padding_top(padding);
        self.widget.set_padding_right(padding);
        self.widget.set_padding_bottom(padding);
        self.widget.set_padding_left(padding);
    }
    pub fn padding(mut self, padding: f32) -> Self {
        self.set_padding(padding);
        self
    }
}

fn minimum_padding(radius: (f32, f32, f32, f32)) -> f32 {
    let (tl, tr, br, bl) = radius;
    let max = tl.max(tr).max(br).max(bl);
    let radius = max * FRAC_1_SQRT_2;
    return radius.floor();
}

impl<W: Geometry> Geometry for WidgetStyle<W> {
    // fn set_width(&mut self, width: f32) {
    //     let border = self.border.1;
    //     self.widget.set_width(width - 2. * border)
    // }
    // fn set_height(&mut self, height: f32) {
    //     let border = self.border.1;
    //     self.widget.set_height(height - 2. * border)
    // }
    fn width(&self) -> f32 {
        self.size.width + 2. * self.border.1
    }
    fn height(&self) -> f32 {
        self.size.height + 2. * self.border.1
    }
}

impl<W: Style> WidgetStyle<W> {
    pub fn set_top_left_radius(&mut self, radius: f32) {
        self.radius.0 = radius;
        self.widget.set_top_left_radius(radius);
        let delta = minimum_padding(self.radius);
        self.widget.padding.0 = self.widget.padding.0.max(delta);
        self.radius.0 += delta;
    }
    pub fn set_top_right_radius(&mut self, radius: f32) {
        self.radius.1 = radius;
        self.widget.set_top_right_radius(radius);
        let delta = minimum_padding(self.radius);
        self.widget.padding.1 = self.widget.padding.1.max(delta);
        self.radius.1 += delta;
    }
    pub fn set_bottom_right_radius(&mut self, radius: f32) {
        self.radius.2 = radius;
        self.widget.set_bottom_right_radius(radius);
        let delta = minimum_padding(self.radius);
        self.widget.padding.2 = self.widget.padding.2.max(delta);
        self.radius.2 += delta;
    }
    pub fn set_bottom_left_radius(&mut self, radius: f32) {
        self.radius.3 = radius;
        self.widget.set_bottom_left_radius(radius);
        let delta = minimum_padding(self.radius);
        self.widget.padding.3 = self.widget.padding.3.max(delta);
        self.radius.3 += delta;
    }
    pub fn set_radius(&mut self, radius: f32) {
        self.widget.set_radius(radius);
        self.radius = (radius, radius, radius, radius);
        let delta = minimum_padding(self.radius);
        self.widget.padding.0 = self.widget.padding.0.max(delta);
        self.widget.padding.1 = self.widget.padding.1.max(delta);
        self.widget.padding.2 = self.widget.padding.2.max(delta);
        self.widget.padding.3 = self.widget.padding.3.max(delta);
        self.radius = (
            radius + delta,
            radius + delta,
            radius + delta,
            radius + delta,
        );
    }
    pub fn radius(mut self, radius: f32) -> Self {
        WidgetStyle::set_radius(&mut self, radius);
        self
    }
    pub fn top_left_radius(mut self, radius: f32) -> Self {
        WidgetStyle::set_top_left_radius(&mut self, radius);
        self
    }
    pub fn top_right_radius(mut self, radius: f32) -> Self {
        WidgetStyle::set_top_right_radius(&mut self, radius);
        self
    }
    pub fn bottom_right_radius(mut self, radius: f32) -> Self {
        WidgetStyle::set_bottom_right_radius(&mut self, radius);
        self
    }
    pub fn bottom_left_radius(mut self, radius: f32) -> Self {
        WidgetStyle::set_bottom_left_radius(&mut self, radius);
        self
    }
}

impl<W> Style for WidgetStyle<W> {
    fn set_radius(&mut self, radius: f32) {
        let delta = minimum_padding(self.radius);
        self.widget.padding.0 = self.widget.padding.0.max(delta);
        self.widget.padding.1 = self.widget.padding.1.max(delta);
        self.widget.padding.2 = self.widget.padding.2.max(delta);
        self.widget.padding.3 = self.widget.padding.3.max(delta);
        self.radius = (radius, radius, radius, radius);
    }
    fn set_top_left_radius(&mut self, radius: f32) {
        let delta = minimum_padding(self.radius);
        self.widget.padding.0 = self.widget.padding.0.max(delta);
        self.radius.0 = radius;
    }
    fn set_top_right_radius(&mut self, radius: f32) {
        let delta = minimum_padding(self.radius);
        self.widget.padding.0 = self.widget.padding.1.max(delta);
        self.radius.1 = radius;
    }
    fn set_bottom_right_radius(&mut self, radius: f32) {
        let delta = minimum_padding(self.radius);
        self.widget.padding.2 = self.widget.padding.2.max(delta);
        self.radius.2 = radius;
    }
    fn set_bottom_left_radius(&mut self, radius: f32) {
        let delta = minimum_padding(self.radius);
        self.widget.padding.2 = self.widget.padding.3.max(delta);
        self.radius.3 = radius;
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
        let (border_texture, border_size) = self.border.clone();
        if let Some(node) = self
            .widget
            .create_node(transform.pre_translate(border_size, border_size))
            .as_option()
        {
            let width = self.size.width;
            let height = self.size.height;
            RenderNode::Decoration {
                node: Box::new(node),
                border: {
                    if border_texture != Texture::Transparent || border_size > 0. {
                        Some(
                            Rectangle::new(width, height)
                                .top_left_radius(self.radius.0)
                                .top_right_radius(self.radius.1)
                                .bottom_right_radius(self.radius.2)
                                .bottom_left_radius(self.radius.3)
                                .border(border_texture, border_size)
                                .to_render_node(transform)
                                .instruction()
                                .unwrap()
                        )
                    } else {
                        None
                    }
                },
                background: 
                    Rectangle::new(width, height)
                        .background(self.background.clone())
                        .top_left_radius(minimum_radius(self.radius.0, border_size))
                        .top_right_radius(minimum_radius(self.radius.1, border_size))
                        .bottom_right_radius(minimum_radius(self.radius.2, border_size))
                        .bottom_left_radius(minimum_radius(self.radius.3, border_size))
                        .to_render_node(transform.pre_translate(border_size, border_size))
                        .instruction()
                        .unwrap()
            }
        } else {
            return RenderNode::None;
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
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        let border_size = self.border.1;
        let size = self
            .widget
            .layout(ctx, &constraints.crop(border_size * 2., border_size * 2.));
        self.size = size.round();
        let (width, height) = self.size.into();
        (width + 2. * border_size, height + 2. * border_size).into()
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
