use crate::scene::*;
use crate::widgets::shapes::rectangle::{minimum_radius, BorderedRectangle, Rectangle};
use crate::widgets::shapes::Style;
use crate::*;
use std::f32::consts::FRAC_1_SQRT_2;
use std::ops::{Deref, DerefMut};
use widgets::Padding;

/// Main type used for styling.
/// Any widget can be wrapped in a WidgetStyle and take advantage of the Style trait.
#[derive(Debug)]
pub struct WidgetStyle<W> {
    background: Rectangle,
    border: BorderedRectangle,
    widget: Positioner<Padding<W>>,
}

impl<W: Geometry> WidgetStyle<W> {
    pub fn new(widget: W) -> Self {
        WidgetStyle {
            background: Rectangle::new(0., 0.),
            border: BorderedRectangle::new(0., 0.),
            widget: Positioner::new(Padding::new(widget)),
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
    pub fn set_border_width(&mut self, width: f32) {
        self.widget.set_coords(width, width);
        self.border.set_border_width(width)
    }
    pub fn border_width(mut self, width: f32) -> Self {
        self.set_border_width(width);
        self
    }
    pub fn set_border_texture(&mut self, texture: impl Into<Texture>) {
        self.border.set_texture(texture);
    }
    pub fn border(mut self, texture: impl Into<Texture>, width: f32) -> Self {
        self.set_border_texture(texture);
        self.set_border_width(width);
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
    fn width(&self) -> f32 {
        self.background.width().max(self.border.width())
    }
    fn height(&self) -> f32 {
        self.background.height().max(self.border.height())
    }
    fn contains(&self, x: f32, y: f32) -> bool {
        self.widget
            .contains(x - self.border.border_width, y - self.border.border_width)
    }
}

impl<W: Style> WidgetStyle<W> {
    pub fn set_top_left_radius(&mut self, radius: f32) {
        let radius = minimum_radius(radius, self.border.border_width);
        self.border.set_top_left_radius(radius);
        self.widget.set_top_left_radius(radius);
        self.background.set_top_left_radius(radius);
        let delta = minimum_padding(self.background.radius);
        self.widget.padding.0 = self.widget.padding.0.max(delta);
        self.background
            .set_top_left_radius((radius - self.border.border_width).max(0.) + delta);
        self.border.set_top_left_radius(radius + delta);
    }
    pub fn set_top_right_radius(&mut self, radius: f32) {
        let radius = minimum_radius(radius, self.border.border_width);
        self.border.set_top_right_radius(radius);
        self.widget.set_top_right_radius(radius);
        self.background.set_top_right_radius(radius);
        let delta = minimum_padding(self.background.radius);
        self.widget.padding.1 = self.widget.padding.1.max(delta);
        self.background
            .set_top_right_radius((radius - self.border.border_width).max(0.) + delta);
        self.border.set_top_right_radius(radius + delta);
    }
    pub fn set_bottom_right_radius(&mut self, radius: f32) {
        let radius = minimum_radius(radius, self.border.border_width);
        self.border.set_bottom_right_radius(radius);
        self.widget.set_bottom_right_radius(radius);
        self.background.set_bottom_right_radius(radius);
        let delta = minimum_padding(self.background.radius);
        self.widget.padding.2 = self.widget.padding.2.max(delta);
        self.background
            .set_bottom_right_radius((radius - self.border.border_width).max(0.) + delta);
        self.border.set_bottom_right_radius(radius + delta);
    }
    pub fn set_bottom_left_radius(&mut self, radius: f32) {
        let radius = minimum_radius(radius, self.border.border_width);
        self.background.radius.3 = radius;
        self.border.radius.3 = radius;
        self.widget.set_bottom_left_radius(radius);
        let delta = minimum_padding(self.background.radius);
        self.widget.padding.3 = self.widget.padding.3.max(delta);
        self.background.set_bottom_left_radius(radius + delta);
        self.border.set_bottom_left_radius(radius + delta);
    }
    pub fn set_radius(&mut self, radius: f32) {
        self.widget.set_radius(radius);
        self.background.radius = (radius, radius, radius, radius);
        let delta = minimum_padding(self.background.radius);
        self.widget.padding.0 = self.widget.padding.0.max(delta);
        self.widget.padding.1 = self.widget.padding.1.max(delta);
        self.widget.padding.2 = self.widget.padding.2.max(delta);
        self.widget.padding.3 = self.widget.padding.3.max(delta);
        self.background
            .set_radius((radius - self.border.border_width).max(0.) + delta);
        self.border.set_radius(radius + delta);
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
        self.background.radius = (radius, radius, radius, radius);
        let delta = minimum_padding(self.background.radius);
        self.widget.padding.0 = self.widget.padding.0.max(delta);
        self.widget.padding.1 = self.widget.padding.1.max(delta);
        self.widget.padding.2 = self.widget.padding.2.max(delta);
        self.widget.padding.3 = self.widget.padding.3.max(delta);
        self.background
            .set_radius((radius - self.border.border_width).max(0.));
        self.border.set_radius(radius);
    }
    fn set_top_left_radius(&mut self, radius: f32) {
        self.background.set_top_left_radius(radius);
        let delta = minimum_padding(self.background.radius);
        self.widget.padding.0 = self.widget.padding.0.max(delta);
        self.background
            .set_top_left_radius((radius - self.border.border_width).max(0.));
        self.border.set_top_left_radius(radius);
    }
    fn set_top_right_radius(&mut self, radius: f32) {
        self.background.set_top_right_radius(radius);
        let delta = minimum_padding(self.background.radius);
        self.widget.padding.0 = self.widget.padding.1.max(delta);
        self.background
            .set_top_right_radius((radius - self.border.border_width).max(0.));
        self.border.set_top_right_radius(radius);
    }
    fn set_bottom_right_radius(&mut self, radius: f32) {
        self.background.set_bottom_right_radius(radius);
        let delta = minimum_padding(self.background.radius);
        self.widget.padding.2 = self.widget.padding.2.max(delta);
        self.background
            .set_bottom_right_radius((radius - self.border.border_width).max(0.));
        self.border.set_bottom_right_radius(radius);
    }
    fn set_bottom_left_radius(&mut self, radius: f32) {
        self.background.set_bottom_left_radius(radius);
        let delta = minimum_padding(self.background.radius);
        self.widget.padding.2 = self.widget.padding.3.max(delta);
        self.background
            .set_bottom_left_radius((radius - self.border.border_width).max(0.));
        self.border.set_bottom_left_radius(radius);
    }
    fn set_texture<B: Into<Texture>>(&mut self, texture: B) {
        self.background.set_texture(texture);
    }
}

impl<D, W: Widget<D>> Widget<D> for WidgetStyle<W> {
    fn draw_scene<'b>(&'b mut self, mut scene: Scene<'_, '_, 'b>) {
        let Coords { x, y } = self.widget.coords();
        if !self.border.texture.is_transparent() || self.border.border_width > 0. {
            if let Some(scene) = scene.apply_border(&self.border) {
                if let Some(scene) = scene.translate(x, y).apply_background(&self.background) {
                    self.widget.deref_mut().draw_scene(scene);
                }
            }
        } else if !self.background.texture.is_transparent() {
            if let Some(scene) = scene.translate(x, y).apply_background(&self.background) {
                self.widget.deref_mut().draw_scene(scene);
            }
        } else {
            self.widget.draw_scene(scene);
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event) -> Damage {
        self.widget.sync(ctx, event)
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        let size = self
            .widget
            .layout(
                ctx,
                &constraints.crop(self.border.border_width * 2., self.border.border_width * 2.),
            )
            .round();
        self.background.set_size(size.width, size.height);
        self.border.set_size(size.width, size.height);
        Size::new(self.width(), self.height())
    }
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
