use crate::*;

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
    pub(crate) inner: W,
    damage: Damage,
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
}

impl<D, W: Widget<D>> Widget<D> for Proxy<W> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        match self.damage {
            Damage::None => {
                RenderNode::None
            }
            _ => {
                self.damage = Damage::None;
                self.inner.create_node(transform)
            }
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        self.damage = self.damage.max(match event {
            Event::Configure(_) | Event::Prepare => {
                Damage::Partial.max(self.inner.sync(ctx, event))
            }
            _ => self.inner.sync(ctx, event),
        });
        self.damage
    }
}

use widgets::shapes::Style;

impl <W: Style> Style for Proxy<W> {
    fn set_background<B: Into<scene::Texture>>(&mut self, texture: B) {
        self.inner.set_background(texture);
    }
    fn set_border_size(&mut self, size: f32) {
        self.inner.set_border_size(size);
    }
    fn set_border_texture<T: Into<scene::Texture>>(&mut self, texture: T) {
        self.inner.set_border_texture(texture);
    }
    fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.inner.set_radius(tl, tr, br, bl);
    }
}

impl<W> Proxy<W> {
    pub fn new(inner: W) -> Self {
        Proxy {
            inner,
            damage: Damage::Partial,
        }
    }
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }
}
pub trait Flex<G>: Geometry + Sized {
    fn with_width(self, width: f32) -> Self;
    fn with_height(self, height: f32) -> Self;
    fn with_size(self, width: f32, height: f32) -> Self;
}

pub trait Wrapped: Sized + Geometry {
    fn ext(self) -> WidgetExt<Self>;
    fn clamp(self) -> WidgetBox<Self>;
    fn child(self) -> Positioner<Proxy<Self>>;
    fn pad(self, padding: f32) -> Padding<Self>;
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

impl<G> Flex<G> for G
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
    fn pad(self, padding: f32) -> Padding<Self> {
        Padding::new(self).even_padding(padding)
    }
    fn clamp(self) -> WidgetBox<Self> {
        WidgetBox::new(self)
    }
    fn ext(self) -> WidgetExt<Self> {
        WidgetExt::new(self)
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
}

impl<D> Widget<D> for Box<dyn Widget<D>> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        self.deref_mut().create_node(transform)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        self.deref_mut().sync(ctx, event)
    }
}
