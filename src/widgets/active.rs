use crate::mail::Mail;
use crate::*;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct Activate<T, W, M, F>
where
    F: for<'d> FnMut(&'d mut Proxy<W>, bool, &'d mut UpdateContext<T>),
{
    cb: F,
    message: M,
    proxy: Proxy<W>,
    active: bool,
    _data: PhantomData<T>,
}

impl<T, W, M, F> Activate<T, W, M, F>
where
    W: Widget<T>,
    F: for<'d> FnMut(&'d mut Proxy<W>, bool, &'d mut UpdateContext<T>),
{
    pub fn new(child: W, message: M, cb: F) -> Self {
        Self {
            cb,
            message,
            active: false,
            proxy: Proxy::new(child),
            _data: PhantomData,
        }
    }
    pub fn active(&self) -> bool {
        self.active
    }
}

impl<T, W, M, F> Widget<T> for Activate<T, W, M, F>
where
    W: Widget<T>,
    T: for<'a, 'b> Mail<'a, &'b M, bool, bool>,
    F: for<'d> FnMut(&'d mut Proxy<W>, bool, &'d mut UpdateContext<T>),
{
    fn draw_scene(&mut self, scene: Scene) {
        self.proxy.draw_scene(scene);
    }
    fn event<'s>(&'s mut self, ctx: &mut UpdateContext<T>, event: Event<'s>) -> Damage {
        self.proxy.event(ctx, event)
    }
    fn update<'s>(&'s mut self, ctx: &mut UpdateContext<T>) -> Damage {
        if let Some(active) = ctx.get(&self.message) {
            self.active = active;
            (self.cb)(&mut self.proxy, active, ctx);
        }
        self.proxy.update(ctx)
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        self.proxy.layout(ctx, constraints)
    }
}

impl<T, W, M, F> Deref for Activate<T, W, M, F>
where
    W: Widget<T>,
    F: for<'d> FnMut(&'d mut Proxy<W>, bool, &'d mut UpdateContext<T>),
{
    type Target = Proxy<W>;
    fn deref(&self) -> &Self::Target {
        &self.proxy
    }
}

impl<T, W, M, F> DerefMut for Activate<T, W, M, F>
where
    W: Widget<T>,
    F: for<'d> FnMut(&'d mut Proxy<W>, bool, &'d mut UpdateContext<T>),
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.proxy
    }
}
