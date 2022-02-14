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
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Size { width, height }
    }
    pub fn round(&self) -> Self {
        Size::new(self.width.round(), self.height.round())
    }
    pub fn ceil(&self) -> Self {
        Size::new(self.width.ceil(), self.height.ceil())
    }
    pub fn floor(&self) -> Self {
        Size::new(self.width.floor(), self.height.floor())
    }
}

impl From<(f32, f32)> for Size {
    fn from((width, height): (f32, f32)) -> Self {
        Size { width, height }
    }
}

impl From<Size> for (f32, f32) {
    fn from(Size { width, height }: Size) -> Self {
        (width, height)
    }
}

impl Default for Size {
    fn default() -> Self {
        Size::new(0., 0.)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoxConstraints {
    minimum: Size,
    maximum: Size,
}

impl Default for BoxConstraints {
    fn default() -> Self {
        Self {
            minimum: Default::default(),
            maximum: Default::default(),
        }
    }
}

impl BoxConstraints {
    pub fn new<S: Into<Size>>(minimum: S, maximum: S) -> Self {
        BoxConstraints {
            minimum: minimum.into(),
            maximum: maximum.into(),
        }
    }
    pub fn loosen(&self) -> Self {
        BoxConstraints {
            minimum: Size::default(),
            maximum: self.maximum,
        }
    }
    pub fn with_max(&self, width: f32, height: f32) -> Self {
        let width = self.maximum_width().min(width);
        let height = self.maximum_height().min(height);
        BoxConstraints {
            minimum: Size::new(
                width.min(self.minimum_width()),
                height.min(self.minimum_height()),
            ),
            maximum: Size::new(width, height),
        }
    }
    pub fn with_min(&self, width: f32, height: f32) -> Self {
        let width = width.max(0.);
        let height = height.max(0.);
        BoxConstraints {
            minimum: Size::new(width, height),
            maximum: Size::new(
                width.max(self.maximum_width()),
                height.max(self.maximum_height()),
            ),
        }
    }
    pub fn crop(&self, dx: f32, dy: f32) -> Self {
        let width = (self.maximum_width() - dx).max(0.);
        let height = (self.maximum_height() - dy).max(0.);
        BoxConstraints {
            minimum: Size::new(
                self.minimum_width().min(width),
                self.minimum_height().min(height),
            ),
            maximum: Size::new(width, height),
        }
    }
    pub fn minimum_width(&self) -> f32 {
        self.minimum.width
    }
    pub fn minimum_height(&self) -> f32 {
        self.minimum.height
    }
    pub fn maximum_width(&self) -> f32 {
        self.maximum.width
    }
    pub fn maximum_height(&self) -> f32 {
        self.maximum.height
    }
}

/// Analog to xdg_shell states
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

/// Track damage and filters events.
#[derive(Debug)]
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
            Event::Keyboard(_) => todo!(),
            Event::Configure(_) | Event::Draw => Damage::Partial.max(self.inner.sync(ctx, event)),
            _ => self.inner.sync(ctx, event),
        });
        self.damage
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        self.inner.layout(ctx, constraints)
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
            damage: Damage::None,
        }
    }
    /// Increment the damage
    pub fn load(&mut self) {
        self.damage = self.damage.max(Damage::Partial);
    }
    /// Returns a mutable reference to the inner type without incrementing the damage.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }
}

/// Additional method to build common widgets
pub trait WidgetExt: Sized + Geometry {
    fn style(self) -> WidgetStyle<Self>;
    fn clamp(self) -> WidgetBox<Self>;
    fn button<D, F>(self, cb: F) -> Button<D, Self, F>
    where
        F: for<'d> FnMut(&'d mut Proxy<Self>, &'d mut SyncContext<D>, Pointer);
}

/// For widgets who's size can be determined at runtime
pub trait GeometryExt: Sized {
    fn set_width(&mut self, width: f32);
    fn set_height(&mut self, height: f32);
    fn set_size(&mut self, width: f32, height: f32) {
        self.set_width(width);
        self.set_height(height);
    }
    fn with_width(mut self, width: f32) -> Self {
        self.set_width(width);
        self
    }
    fn with_height(mut self, height: f32) -> Self {
        self.set_height(height);
        self
    }
    fn with_size(mut self, width: f32, height: f32) -> Self {
        self.set_size(width, height);
        self
    }
}

/// Implement PartialEq across different types.
///
/// It is automatically implemented if a type implements PartialEq
pub trait DynEq {
    fn same(&self, other: &dyn std::any::Any) -> bool;
    fn not_same(&self, other: &dyn std::any::Any) -> bool {
        !self.same(other)
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

impl<W> WidgetExt for W
where
    W: Geometry,
{
    fn clamp(self) -> WidgetBox<Self> {
        WidgetBox::new(self)
    }
    fn style(self) -> WidgetStyle<Self> {
        WidgetStyle::new(self)
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
}

impl<D> Widget<D> for Box<dyn Widget<D>> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        self.deref_mut().create_node(transform)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        self.deref_mut().sync(ctx, event)
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        self.deref_mut().layout(ctx, constraints)
    }
}
