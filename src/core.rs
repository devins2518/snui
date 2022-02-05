use crate::*;

/// Cursor mimetype
pub enum Cursor {
    Arrow,
    TopLeftCorner,
    TopRightCorner,
    BottomRightCorner,
    BottomLeftCorner,
    TopSide,
    BottomSide,
    RightSide,
    LeftSide,
    Cross,
    PointCenter,
    PointLeft,
    ColumnResize,
    RowResize,
    CrossHair,
    DragDropMove,
    DragDropNone,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Draft,
    Help,
    Kill,
    Blocked,
    Hand,
    OpenHand,
    Wait,
}

impl Cursor {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Arrow => "arrow",
            Self::TopLeftCorner => "top_left_corner",
            Self::TopRightCorner => "top_right_corner",
            Self::BottomRightCorner => "bottom_right_corner",
            Self::BottomLeftCorner => "bottom_left_corner",
            Self::TopSide => "top_side",
            Self::BottomSide => "bottom_side",
            Self::RightSide => "right_side",
            Self::LeftSide => "left_side",
            Self::Cross => "cross",
            Self::PointCenter => "center_ptr",
            Self::PointLeft => "left_ptr",
            Self::ColumnResize => "",
            Self::RowResize => "",
            Self::CrossHair => "crosshair",
            Self::DragDropMove => "",
            Self::DragDropNone => "",
            Self::ArrowUp => "up_arrow",
            Self::ArrowDown => "down_arror",
            Self::ArrowLeft => "left_arrow",
            Self::ArrowRight => "right_arrow",
            Self::Draft => "draft_large",
            Self::Help => "help",
            Self::Kill => "kill",
            Self::Blocked => "block",
            Self::Hand => "hand1",
            Self::OpenHand => "hand2",
            Self::Wait => "wait",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WindowState {
    Maximized,
    Resizing,
    Fullscreen,
    /// Client window decorations should be painted as if the window is active.
    Activated,
    TiledLeft,
    TiledRight,
    TiledBottom,
    TiledTop,
}

pub struct Proxy<W> {
    damage: Damage,
    entered: bool,
    pub(crate) inner: W,
}

impl<W> Deref for Proxy<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<W> DerefMut for Proxy<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.damage = self.damage.max(Damage::Partial);
        &mut self.inner
    }
}

impl<W: Geometry> Geometry for Proxy<W> {
    fn width(&self) -> f32 {
        self.inner.width()
    }
    fn height(&self) -> f32 {
        self.inner.height()
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.inner.set_width(width)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.inner.set_height(height)
    }
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        self.inner.set_size(width, height)
    }
    fn maximum_height(&self) -> f32 {
        self.inner.maximum_height()
    }
    fn minimum_height(&self) -> f32 {
        self.inner.minimum_height()
    }
    fn maximum_width(&self) -> f32 {
        self.inner.maximum_width()
    }
    fn minimum_width(&self) -> f32 {
        self.inner.minimum_width()
    }
}

impl<D, W: Widget<D>> Widget<D> for Proxy<W> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        match self.damage {
            Damage::None => RenderNode::None,
            _ => {
                self.damage = Damage::None;
                self.inner.create_node(transform)
            }
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        self.damage = self.damage.max(match event {
            Event::Pointer(x, y, _) => {
                if self.contains(x, y) {
                    self.entered = true;
                    self.inner.sync(ctx, event)
                } else if self.entered {
                    let e = self.inner.sync(ctx, event);
                    self.entered = self.contains(x, y) || e.is_some();
                    e
                } else {
                    Damage::None
                }
            }
            Event::Configure(_) | Event::Prepare => {
                Damage::Partial.max(self.inner.sync(ctx, event))
            }
            _ => self.inner.sync(ctx, event),
        });
        self.damage
    }
}

use widgets::shapes::Style;

impl<W: Style> Style for Proxy<W> {
    fn set_background<B: Into<scene::Texture>>(&mut self, texture: B) {
        self.inner.set_background(texture);
    }
    fn set_border_size(&mut self, size: f32) {
        self.inner.set_border_size(size);
    }
    fn set_border_texture<T: Into<scene::Texture>>(&mut self, texture: T) {
        self.inner.set_border_texture(texture);
    }
    fn set_radius_top_left(&mut self, radius: f32) {
        self.inner.set_radius_top_left(radius);
    }
    fn set_radius_top_right(&mut self, radius: f32) {
        self.inner.set_radius_top_right(radius);
    }
    fn set_radius_bottom_right(&mut self, radius: f32) {
        self.inner.set_radius_bottom_right(radius);
    }
    fn set_radius_bottom_left(&mut self, radius: f32) {
        self.inner.set_radius_bottom_left(radius);
    }
}

use widgets::scroll::Scrollable;

impl<W: Scrollable> Scrollable for Proxy<W> {
    fn forward(&mut self, step: Option<f32>) {
        self.inner.forward(step)
    }
    fn backward(&mut self, step: Option<f32>) {
        self.inner.backward(step)
    }
    fn inner_height(&self) -> f32 {
        self.inner.inner_height()
    }
    fn inner_width(&self) -> f32 {
        self.inner.inner_width()
    }
    fn orientation(&self) -> Orientation {
        self.inner.orientation()
    }
    fn position(&self) -> f32 {
        self.inner.position()
    }
}

impl<W> Proxy<W> {
    pub fn new(inner: W) -> Self {
        Proxy {
            inner,
            entered: false,
            damage: Damage::Partial,
        }
    }
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }
}
pub trait GeometryExt<G>: Geometry + Sized {
    fn with_width(self, width: f32) -> Self;
    fn with_height(self, height: f32) -> Self;
    fn with_size(self, width: f32, height: f32) -> Self;
}

pub trait Wrapped: Sized + Geometry {
    fn style(self) -> WidgetStyle<Self>;
    fn clamp(self) -> WidgetBox<Self>;
    fn child(self) -> Positioner<Proxy<Self>>;
    fn button<D, F>(self, cb: F) -> Button<D, Self, F>
    where
        F: for<'d> FnMut(&'d mut Proxy<Self>, &'d mut SyncContext<D>, Pointer);
}

pub trait DynEq {
    fn same(&self, other: &dyn std::any::Any) -> bool;
    fn not_same(&self, other: &dyn std::any::Any) -> bool {
        !self.same(other)
    }
}

impl<G> GeometryExt<G> for G
where
    G: Geometry,
{
    fn with_width(mut self, width: f32) -> Self {
        let _ = self.set_width(width);
        self
    }
    fn with_height(mut self, height: f32) -> Self {
        let _ = self.set_height(height);
        self
    }
    fn with_size(mut self, width: f32, height: f32) -> Self {
        let _ = self.set_size(width, height);
        self
    }
}

impl<T> DynEq for T
where
    T: PartialEq + 'static,
{
    fn same(&self, other: &dyn std::any::Any) -> bool {
        if let Some(other) = other.downcast_ref::<T>() {
            return self.eq(other);
        }
        false
    }
}

impl<W> Wrapped for W
where
    W: Geometry,
{
    fn clamp(self) -> WidgetBox<Self> {
        WidgetBox::new(self)
    }
    fn style(self) -> WidgetStyle<Self> {
        WidgetStyle::new(self)
    }
    fn child(self) -> Positioner<Proxy<Self>> {
        Positioner::new(Proxy::new(self))
    }
    fn button<D, F>(self, cb: F) -> Button<D, Self, F>
    where
        F: for<'d> FnMut(&'d mut Proxy<Self>, &'d mut SyncContext<D>, Pointer),
    {
        Button::new(self, cb)
    }
}

impl<D> Geometry for Box<dyn Widget<D>> {
    fn height(&self) -> f32 {
        self.as_ref().height()
    }
    fn width(&self) -> f32 {
        self.as_ref().width()
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.as_mut().set_width(width)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.as_mut().set_height(height)
    }
    fn maximum_height(&self) -> f32 {
        self.as_ref().maximum_height()
    }
    fn minimum_height(&self) -> f32 {
        self.as_ref().minimum_height()
    }
    fn maximum_width(&self) -> f32 {
        self.as_ref().maximum_width()
    }
    fn minimum_width(&self) -> f32 {
        self.as_ref().minimum_width()
    }
}

impl<D> Widget<D> for Box<dyn Widget<D>> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        self.deref_mut().create_node(transform)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        self.deref_mut().sync(ctx, event)
    }
}
