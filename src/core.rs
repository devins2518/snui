use crate::*;

/// Cursor mimetype
pub enum Cursor {
    Arrow,
    TopLeftCorner,
    TopRightCorner,
    BottomRightCorner,
    BottomLeftCorner,
    Exchange,
    TopSide,
    BottomSide,
    RightSide,
    LeftSide,
    Cross,
    Cursor,
    Mouse,
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
    Sizing,
    Blocked,
    Hand,
    OpenHand,
    Watch,
    Wait,
}

impl Cursor {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Arrow => "arrow",
            Self::Cursor => "cursor",
            Self::TopLeftCorner => "top_left_corner",
            Self::TopRightCorner => "top_right_corner",
            Self::BottomRightCorner => "bottom_right_corner",
            Self::BottomLeftCorner => "bottom_left_corner",
            Self::Exchange => "exchange",
            Self::TopSide => "top_side",
            Self::BottomSide => "bottom_side",
            Self::RightSide => "right_side",
            Self::LeftSide => "left_side",
            Self::Cross => "cross",
            Self::Mouse => "mouse",
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
            Self::Sizing => "sizing",
            Self::Blocked => "block",
            Self::Hand => "hand1",
            Self::OpenHand => "hand2",
            Self::Wait => "wait",
            Self::Watch => "watch",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Geometry for Size {
    fn width(&self) -> f32 {
        self.width
    }
    fn height(&self) -> f32 {
        self.height
    }
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
    pub fn maximum(&self) -> Size {
        self.maximum
    }
    pub fn minimum(&self) -> Size {
        self.minimum
    }
    pub fn is_default(&self) -> bool {
        self.minimum.width == 0.
            && self.minimum.height == 0.
            && self.maximum.width == 0.
            && self.maximum.height == 0.
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
    size: Size,
    damage: Damage,
    entered: bool,
    pub(crate) inner: W,
}

impl<W> Proxy<W> {
    pub fn new(inner: W) -> Self {
        Proxy {
            inner,
            size: Size::default(),
            entered: false,
            damage: Damage::Partial,
        }
    }
    /// Increment the damage
    pub fn upgrade(&mut self) {
        self.damage = self.damage.max(Damage::Partial);
    }
    /// Returns a mutable reference to the inner type without incrementing the damage.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }
}

impl<W> Deref for Proxy<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<W> DerefMut for Proxy<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.damage.upgrade();
        &mut self.inner
    }
}

impl<W> Geometry for Proxy<W> {
    fn width(&self) -> f32 {
        self.size.width
    }
    fn height(&self) -> f32 {
        self.size.height
    }
}

impl<D, W: Widget<D>> Widget<D> for Proxy<W> {
    fn draw_scene(&mut self, scene: Scene) {
        if self.damage.is_some() || scene.damage_state() {
            self.inner.draw_scene(scene)
        }
        self.damage = Damage::None
    }
    fn update<'s>(&'s mut self, ctx: &mut SyncContext<D>) -> Damage {
        self.damage = self.damage.max(self.inner.update(ctx));
        self.damage
    }
    fn event<'s>(&'s mut self, ctx: &mut SyncContext<D>, event: Event<'s>) -> Damage {
        self.damage = self.damage.max(match event {
            Event::Pointer(MouseEvent { ref position, .. }) => {
                if self.contains(position) {
                    if self.entered {
                        self.inner.event(ctx, event)
                    } else {
                        self.entered = true;
                        self.inner
                            .event(ctx, MouseEvent::new(*position, Pointer::Enter))
                    }
                } else if self.entered {
                    let damage = self.inner.event(ctx, event);
                    self.entered = self.contains(position) || damage.is_some();
                    if !self.entered {
                        self.inner
                            .event(ctx, MouseEvent::new(*position, Pointer::Leave))
                    } else {
                        damage
                    }
                } else {
                    Damage::None
                }
            }
            Event::Keyboard(_) => todo!(),
            Event::Configure => {
                if ctx
                    .window()
                    .get_state()
                    .iter()
                    .any(|state| matches!(state, WindowState::Resizing))
                {
                    Damage::Partial.max(self.inner.event(ctx, event))
                } else {
                    self.inner.event(ctx, event)
                }
            }
            _ => self.inner.event(ctx, event),
        });
        self.damage
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        if self.damage.is_some() || ctx.force {
            self.size = self.inner.layout(ctx, constraints);
        }
        self.size
    }
}

use scene::Texture;
use widgets::shapes::Style;
use widgets::Padding;

/// Additional method to help build common widgets.
pub trait WidgetExt<T>: Widget<T> + Sized {
    fn clamp(self) -> WidgetBox<T, Self> {
        WidgetBox::new(self)
    }
    fn background(self, background: impl Into<Texture>) -> WidgetStyle<T, Self> {
        Style::texture(WidgetStyle::new(self), background)
    }
    fn border(self, texture: impl Into<Texture>, width: f32) -> WidgetStyle<T, Self> {
        WidgetStyle::new(self).border(texture, width)
    }
    fn padding(self, padding: f32) -> Padding<T, Self> {
        Padding::new(self).padding(padding)
    }
    fn padding_top(self, padding: f32) -> Padding<T, Self> {
        Padding::new(self).padding_top(padding)
    }
    fn padding_right(self, padding: f32) -> Padding<T, Self> {
        Padding::new(self).padding_right(padding)
    }
    fn padding_bottom(self, padding: f32) -> Padding<T, Self> {
        Padding::new(self).padding_bottom(padding)
    }
    fn padding_left(self, padding: f32) -> Padding<T, Self> {
        Padding::new(self).padding_left(padding)
    }
    fn with_min_width(self, width: f32) -> WidgetBox<T, Self> {
        WidgetBox::new(self)
            .constraint(widgets::Constraint::Downward)
            .with_width(width)
    }
    fn with_min_height(self, height: f32) -> WidgetBox<T, Self> {
        WidgetBox::new(self)
            .constraint(widgets::Constraint::Downward)
            .with_height(height)
    }
    fn with_min_size(self, width: f32, height: f32) -> WidgetBox<T, Self> {
        WidgetBox::new(self)
            .constraint(widgets::Constraint::Downward)
            .with_size(width, height)
    }
    fn with_max_width(self, width: f32) -> WidgetBox<T, Self> {
        WidgetBox::new(self)
            .constraint(widgets::Constraint::Upward)
            .with_width(width)
    }
    fn with_max_height(self, height: f32) -> WidgetBox<T, Self> {
        WidgetBox::new(self)
            .constraint(widgets::Constraint::Upward)
            .with_height(height)
    }
    fn with_max_size(self, width: f32, height: f32) -> WidgetBox<T, Self> {
        WidgetBox::new(self)
            .constraint(widgets::Constraint::Upward)
            .with_size(width, height)
    }
    fn with_fixed_width(self, width: f32) -> WidgetBox<T, Self> {
        WidgetBox::new(self)
            .constraint(widgets::Constraint::Fixed)
            .with_width(width)
    }
    fn with_fixed_height(self, height: f32) -> WidgetBox<T, Self> {
        WidgetBox::new(self)
            .constraint(widgets::Constraint::Fixed)
            .with_height(height)
    }
    fn with_fixed_size(self, width: f32, height: f32) -> WidgetBox<T, Self> {
        WidgetBox::new(self)
            .constraint(widgets::Constraint::Fixed)
            .with_size(width, height)
    }
    // TO-DO
    // This should be a macro
    fn button<F>(self, cb: F) -> Proxy<Button<T, Self, F>>
    where
        Self: Widget<T>,
        F: for<'d> FnMut(&'d mut Proxy<Self>, &'d mut SyncContext<T>, Pointer),
    {
        Button::new(self, cb)
    }
}

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

impl<D, W> WidgetExt<D> for W where W: Widget<D> {}

impl<D> Widget<D> for Box<dyn Widget<D>> {
    fn draw_scene(&mut self, scene: Scene) {
        self.deref_mut().draw_scene(scene)
    }
    fn update<'s>(&'s mut self, ctx: &mut SyncContext<D>) -> Damage {
        self.deref_mut().update(ctx)
    }
    fn event<'s>(&'s mut self, ctx: &mut SyncContext<D>, event: Event<'s>) -> Damage {
        self.deref_mut().event(ctx, event)
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        self.deref_mut().layout(ctx, constraints)
    }
}
