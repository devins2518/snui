use crate::*;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct Proxy<R, W>
where
    W: Widget<R>,
{
    child: W,
    damage: Damage,
    queue_draw: bool,
    _request: PhantomData<R>,
}

impl<R, W: Widget<R>> Deref for Proxy<R, W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl<R, W: Widget<R>> DerefMut for Proxy<R, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.damage = self.damage.max(Damage::Some);
        &mut self.child
    }
}

impl<R, W: Widget<R>> Geometry for Proxy<R, W> {
    fn width(&self) -> f32 {
        self.child.width()
    }
    fn height(&self) -> f32 {
        self.child.height()
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.child.set_width(width)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.child.set_height(height)
    }
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        self.child.set_size(width, height)
    }
}

impl<R, W: Widget<R>> Widget<R> for Proxy<R, W> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        if self.queue_draw || self.damage.is_some() {
            self.damage = Damage::None;
            return self.child.create_node(x, y);
        }
        RenderNode::None
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<R>, event: &Event<'d, R>) -> Damage {
        self.damage = self.damage.max(self.child.sync(ctx, event));
        self.queue_draw = self.damage.is_some() || event.is_frame();
        self.damage
    }
}

impl<R, W: Widget<R>> Proxy<R, W> {
    pub fn new(child: W) -> Self {
        Proxy {
            child,
            queue_draw: true,
            damage: Damage::Some,
            _request: PhantomData,
        }
    }
}

pub struct Button<R, W, F>
where
    W: Widget<R>,
    F: for<'d> FnMut(&'d mut Proxy<R, W>, &'d mut SyncContext<R>, Pointer),
{
    focused: bool,
    proxy: Proxy<R, W>,
    cb: F,
}

impl<R, W, F> Button<R, W, F>
where
    W: Widget<R>,
    F: for<'d> FnMut(&'d mut Proxy<R, W>, &'d mut SyncContext<R>, Pointer),
{
    pub fn new(child: W, cb: F) -> Self {
        Self {
            proxy: Proxy::new(child),
            focused: false,
            cb,
        }
    }
}

impl<R, W, F> Geometry for Button<R, W, F>
where
    W: Widget<R>,
    F: for<'d> FnMut(&'d mut Proxy<R, W>, &'d mut SyncContext<R>, Pointer),
{
    fn width(&self) -> f32 {
        self.proxy.width()
    }
    fn height(&self) -> f32 {
        self.proxy.height()
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.proxy.set_width(width)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.proxy.set_height(height)
    }
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        self.proxy.set_size(width, height)
    }
}

impl<R, W, F> Widget<R> for Button<R, W, F>
where
    W: Widget<R>,
    F: for<'d> FnMut(&'d mut Proxy<R, W>, &'d mut SyncContext<R>, Pointer),
{
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        self.proxy.create_node(x, y)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<R>, event: &Event<'d, R>) -> Damage {
        if let Event::Pointer(x, y, pointer) = event {
            if self.contains(*x, *y) {
                if self.focused {
                    (self.cb)(&mut self.proxy, ctx, *pointer);
                } else {
                    self.focused = true;
                    (self.cb)(&mut self.proxy, ctx, Pointer::Enter);
                }
            } else if self.focused {
                self.focused = false;
                (self.cb)(&mut self.proxy, ctx, Pointer::Leave);
            }
        }
        self.proxy.sync(ctx, event)
    }
}

impl<R, W, F> Deref for Button<R, W, F>
where
    W: Widget<R>,
    F: for<'d> FnMut(&'d mut Proxy<R, W>, &'d mut SyncContext<R>, Pointer),
{
    type Target = Proxy<R, W>;
    fn deref(&self) -> &Self::Target {
        &self.proxy
    }
}

impl<R, W, F> DerefMut for Button<R, W, F>
where
    W: Widget<R>,
    F: for<'d> FnMut(&'d mut Proxy<R, W>, &'d mut SyncContext<R>, Pointer),
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.proxy
    }
}
