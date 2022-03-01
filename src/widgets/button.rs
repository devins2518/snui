use crate::*;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct Button<T, W, F>
where
    F: for<'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext<T>, Pointer),
{
    cb: F,
    proxy: Proxy<W>,
    _data: PhantomData<T>,
}

impl<T, W, F> Button<T, W, F>
where
    W: Widget<T>,
    F: for<'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext<T>, Pointer),
{
    pub fn new(child: W, cb: F) -> Proxy<Self> {
        Proxy::new(Self {
            cb,
            proxy: Proxy::new(child),
            _data: PhantomData,
        })
    }
}

impl<T, W, F> Widget<T> for Button<T, W, F>
where
    W: Widget<T>,
    F: for<'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext<T>, Pointer),
{
    fn draw_scene(&mut self, scene: Scene) {
        self.proxy.draw_scene(scene);
    }
    fn event<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
        if let Event::Pointer(MouseEvent { pointer, .. }) = event {
            (self.cb)(&mut self.proxy, ctx, pointer);
        }
        self.proxy.event(ctx, event)
    }
    fn update<'s>(&'s mut self, ctx: &mut SyncContext<T>) -> Damage {
        self.proxy.update(ctx)
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        self.proxy.layout(ctx, constraints)
    }
}

impl<T, W, F> Deref for Button<T, W, F>
where
    W: Widget<T>,
    F: for<'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext<T>, Pointer),
{
    type Target = Proxy<W>;
    fn deref(&self) -> &Self::Target {
        &self.proxy
    }
}

impl<T, W, F> DerefMut for Button<T, W, F>
where
    W: Widget<T>,
    F: for<'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext<T>, Pointer),
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.proxy
    }
}
