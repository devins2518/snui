use crate::widgets::shapes::Style;
use crate::*;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct Button<D, W, F>
where
    F: for<'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext<D>, Pointer),
{
    cb: F,
    entered: bool,
    proxy: Proxy<W>,
    _data: PhantomData<D>,
}

impl<D, W, F> Button<D, W, F>
where
    F: for<'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext<D>, Pointer),
{
    pub fn new(child: W, cb: F) -> Self {
        Self {
            cb,
            entered: false,
            proxy: Proxy::new(child),
            _data: PhantomData,
        }
    }
}

impl<D, W, F> Geometry for Button<D, W, F>
where
    W: Geometry,
    F: for<'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext<D>, Pointer),
{
    fn width(&self) -> f32 {
        self.proxy.width()
    }
    fn height(&self) -> f32 {
        self.proxy.height()
    }
}

impl<D, W, F> Widget<D> for Button<D, W, F>
where
    W: Widget<D>,
    F: for<'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext<D>, Pointer),
{
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        self.proxy.create_node(transform)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        if let Event::Pointer(x, y, pointer) = event {
            if self.contains(x, y) {
                if self.entered {
                    (self.cb)(&mut self.proxy, ctx, pointer);
                } else {
                    self.entered = true;
                    (self.cb)(&mut self.proxy, ctx, Pointer::Enter);
                }
            } else if self.entered {
                self.entered = false;
                (self.cb)(&mut self.proxy, ctx, Pointer::Leave);
            }
        }
        self.proxy.sync(ctx, event)
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        self.proxy.layout(ctx, constraints)
    }
}

impl<D, W, F> Style for Button<D, W, F>
where
    W: Style,
    F: for<'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext<D>, Pointer),
{
    fn set_background<B: Into<scene::Texture>>(&mut self, background: B) {
        self.proxy.set_background(background);
    }
    fn set_border_texture<T: Into<scene::Texture>>(&mut self, texture: T) {
        self.proxy.set_border_texture(texture);
    }
    fn set_border_size(&mut self, size: f32) {
        self.proxy.set_border_size(size);
    }
    fn set_top_left_radius(&mut self, radius: f32) {
        self.proxy.set_top_left_radius(radius);
    }
    fn set_top_right_radius(&mut self, radius: f32) {
        self.proxy.set_top_right_radius(radius);
    }
    fn set_bottom_right_radius(&mut self, radius: f32) {
        self.proxy.set_bottom_right_radius(radius);
    }
    fn set_bottom_left_radius(&mut self, radius: f32) {
        self.proxy.set_radius(radius);
    }
}

impl<D, W, F> Deref for Button<D, W, F>
where
    W: Widget<D>,
    F: for<'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext<D>, Pointer),
{
    type Target = Proxy<W>;
    fn deref(&self) -> &Self::Target {
        &self.proxy
    }
}

impl<D, W, F> DerefMut for Button<D, W, F>
where
    W: Widget<D>,
    F: for<'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext<D>, Pointer),
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.proxy
    }
}
