use crate::scene::*;
use crate::widgets::shapes::rectangle::{BorderedRectangle, Rectangle};
use crate::widgets::shapes::Style;
use crate::*;
use std::f32::consts::FRAC_1_SQRT_2;
use std::ops::{Deref, DerefMut};
use widgets::Padding;

/// Main type used for styling.
/// Any widget can be wrapped in a WidgetStyle and take advantage of the Style trait.
#[derive(Debug)]
pub struct WidgetStyle<T, W: Widget<T>> {
    contained: bool,
    radius: [f32; 4],
    background: Rectangle,
    border: BorderedRectangle,
    widget: Positioner<Padding<T, W>>,
}

impl<T, W: Widget<T>> WidgetStyle<T, W> {
    pub fn new(widget: W) -> Self {
        WidgetStyle {
            contained: false,
            radius: [0.; 4],
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
    pub fn set_border(&mut self, texture: impl Into<Texture>, width: f32) {
        self.set_border_texture(texture);
        self.set_border_width(width);
    }
    pub fn border_width(mut self, width: f32) -> Self {
        self.set_border_width(width);
        self
    }
    pub fn set_border_texture(&mut self, texture: impl Into<Texture>) {
        self.border.set_texture(texture);
    }
    pub fn border(mut self, texture: impl Into<Texture>, width: f32) -> Self {
        self.set_border(texture, width);
        self
    }
    /// A promise that the widget's content will not exceed the boundaries.
    /// This enables some optimization to avoid redrawing the whole widget
    /// if it's child doesn't have enough padding
    pub fn contained(mut self) -> Self {
        self.contained = true;
        self
    }
}

fn minimum_padding(radius: [f32; 4]) -> f32 {
    let [tl, tr, br, bl] = radius;
    let max = tl.max(tr).max(br).max(bl);
    let radius = max * FRAC_1_SQRT_2;
    radius.floor()
}

impl<T, W: Widget<T>> Geometry for WidgetStyle<T, W> {
    fn width(&self) -> f32 {
        self.background.width().max(self.border.width())
    }
    fn height(&self) -> f32 {
        self.background.height().max(self.border.height())
    }
}

impl<T, W: Widget<T> + Style> WidgetStyle<T, W> {
    pub fn set_top_left_radius(&mut self, radius: f32) {
        self.radius[0] = radius;
        let [top, _, _, left] = self.widget.padding;
        self.border.set_top_left_radius(radius + top.max(left));
        self.background.set_top_left_radius(radius + top.max(left));
        self.widget.set_top_left_radius(radius);
    }
    pub fn set_top_right_radius(&mut self, radius: f32) {
        self.radius[1] = radius;
        let [top, right, _, _] = self.widget.padding;
        self.border.set_top_right_radius(radius + top.max(right));
        self.background
            .set_top_right_radius(radius + top.max(right));
        self.widget.set_top_right_radius(radius);
    }
    pub fn set_bottom_right_radius(&mut self, radius: f32) {
        self.radius[2] = radius;
        let [_, right, bottom, _] = self.widget.padding;
        self.border
            .set_bottom_right_radius(radius + bottom.max(right));
        self.background
            .set_bottom_right_radius(radius + bottom.max(right));
        self.widget.set_bottom_right_radius(radius);
    }
    pub fn set_bottom_left_radius(&mut self, radius: f32) {
        self.radius[3] = radius;
        let [_, _, bottom, left] = self.widget.padding;
        self.border
            .set_bottom_left_radius(radius + bottom.max(left));
        self.background
            .set_bottom_left_radius(radius + bottom.max(left));
        self.widget.set_bottom_left_radius(radius);
    }
    pub fn set_radius(&mut self, radius: f32) {
        WidgetStyle::set_top_left_radius(self, radius);
        WidgetStyle::set_top_right_radius(self, radius);
        WidgetStyle::set_bottom_left_radius(self, radius);
        WidgetStyle::set_bottom_right_radius(self, radius);
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

impl<T, W: Widget<T>> Style for WidgetStyle<T, W> {
    fn set_radius(&mut self, radius: f32) {
        self.radius[0] = radius;
        let delta = minimum_padding(self.radius);
        self.widget.padding[1] = self.widget.padding[1].max(delta);
        self.widget.padding[3] = self.widget.padding[3].max(delta);
        self.background
            .set_radius((radius - self.border.border_width).max(0.));
        self.border.set_radius(radius);
    }
    fn set_top_left_radius(&mut self, radius: f32) {
        self.radius[1] = radius;
        let delta = minimum_padding(self.radius);
        self.widget.padding[3] = self.widget.padding[0].max(delta);
        self.background
            .set_top_left_radius((radius - self.border.border_width).max(0.));
        self.border.set_top_left_radius(radius);
    }
    fn set_top_right_radius(&mut self, radius: f32) {
        self.radius[2] = radius;
        let delta = minimum_padding(self.radius);
        self.widget.padding[1] = self.widget.padding[1].max(delta);
        self.background
            .set_top_right_radius((radius - self.border.border_width).max(0.));
        self.border.set_top_right_radius(radius);
    }
    fn set_bottom_right_radius(&mut self, radius: f32) {
        self.radius[3] = radius;
        let delta = minimum_padding(self.radius);
        self.widget.padding[1] = self.widget.padding[2].max(delta);
        self.background
            .set_bottom_right_radius((radius - self.border.border_width).max(0.));
        self.border.set_bottom_right_radius(radius);
    }
    fn set_bottom_left_radius(&mut self, radius: f32) {
        self.background.set_bottom_left_radius(radius);
        let delta = minimum_padding(self.background.radius);
        self.widget.padding[3] = self.widget.padding[3].max(delta);
        self.background
            .set_bottom_left_radius((radius - self.border.border_width).max(0.));
        self.border.set_bottom_left_radius(radius);
    }
    fn set_texture<B: Into<Texture>>(&mut self, texture: B) {
        self.background.set_texture(texture);
    }
}

impl<T, W: Widget<T>> Widget<T> for WidgetStyle<T, W> {
    fn draw_scene(&mut self, mut scene: Scene) {
        let Coords { x, y } = self.widget.coords();
        if !self.contained {
            let [tl, tr, br, bl] = self.radius;
            let [_, right, _, left] = self.widget.padding;
            if left < (tl.max(bl) * FRAC_1_SQRT_2).floor()
                || right < (br.max(tr) * FRAC_1_SQRT_2).floor()
            {
                scene = scene.damage(Size::new(self.width(), self.height()));
            }
        }
        if let Some(scene) = scene.apply_border(&self.border) {
            if let Some(scene) = scene.translate(x, y).apply_background(&self.background) {
                self.widget.deref_mut().draw_scene(scene);
            };
        };
    }
    fn update<'s>(&'s mut self, ctx: &mut UpdateContext<T>) -> Damage {
        self.widget.update(ctx)
    }
    fn event<'s>(&'s mut self, ctx: &mut UpdateContext<T>, event: Event<'s>) -> Damage {
        self.widget.event(ctx, event)
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

impl<T, W: Widget<T>> Deref for WidgetStyle<T, W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        self.widget.deref()
    }
}

impl<T, W: Widget<T>> DerefMut for WidgetStyle<T, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.widget.deref_mut()
    }
}
