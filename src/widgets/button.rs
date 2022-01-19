use crate::*;
use crate::widgets::shapes::Style;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct Proxy<M, W>
where
    W: Widget<M>,
{
    child: W,
    damage: Damage,
    _request: PhantomData<M>,
}

impl<M, W: Widget<M>> Deref for Proxy<M, W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl<M, W: Widget<M>> DerefMut for Proxy<M, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.damage = self.damage.max(Damage::Partial);
        &mut self.child
    }
}

impl<M, W: Widget<M>> Geometry for Proxy<M, W> {
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

impl<M, W: Widget<M>> Widget<M> for Proxy<M, W> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        if self.damage.is_some() {
            self.damage = Damage::None;
            return self.child.create_node(x, y);
        }
        RenderNode::None
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage {
        self.damage = self.damage.max(match event {
            Event::Configure(_) => {
                Damage::Partial.max(self.child.sync(ctx, event))
            }
            _ => self.child.sync(ctx, event)
        });
        self.damage
    }
}

impl<M, W: Widget<M>> Proxy<M, W> {
    pub fn new(child: W) -> Self {
        Proxy {
            child,
            damage: Damage::Partial,
            _request: PhantomData,
        }
    }
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.child
    }
}

pub struct Button<M, W, F>
where
    W: Widget<M>,
    F: for<'d> FnMut(&'d mut Proxy<M, W>, &'d mut SyncContext<M>, Pointer),
{
    focused: bool,
    proxy: Proxy<M, W>,
    cb: F,
}

impl<M, W, F> Button<M, W, F>
where
    W: Widget<M>,
    F: for<'d> FnMut(&'d mut Proxy<M, W>, &'d mut SyncContext<M>, Pointer),
{
    pub fn new(child: W, cb: F) -> Self {
        Self {
            proxy: Proxy::new(child),
            focused: false,
            cb,
        }
    }
}

impl<M, W, F> Geometry for Button<M, W, F>
where
    W: Widget<M>,
    F: for<'d> FnMut(&'d mut Proxy<M, W>, &'d mut SyncContext<M>, Pointer),
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

impl<M, W, F> Widget<M> for Button<M, W, F>
where
    W: Widget<M>,
    F: for<'d> FnMut(&'d mut Proxy<M, W>, &'d mut SyncContext<M>, Pointer),
{
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        self.proxy.create_node(x, y)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage {
        if let Event::Pointer(x, y, pointer) = event {
            if self.contains(x, y) {
                if self.focused {
                    (self.cb)(&mut self.proxy, ctx, pointer);
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

impl<M, W, F> Style for Button<M, W, F>
where
    W: Widget<M> + Style,
    F: for<'d> FnMut(&'d mut Proxy<M, W>, &'d mut SyncContext<M>, Pointer),
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
    fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.proxy.set_radius(tl, tr, br, bl);
    }
    fn set_even_radius(&mut self, radius: f32) {
        self.proxy.set_even_radius(radius);
    }
}

impl<M, W, F> Deref for Button<M, W, F>
where
    W: Widget<M>,
    F: for<'d> FnMut(&'d mut Proxy<M, W>, &'d mut SyncContext<M>, Pointer),
{
    type Target = Proxy<M, W>;
    fn deref(&self) -> &Self::Target {
        &self.proxy
    }
}

impl<M, W, F> DerefMut for Button<M, W, F>
where
    W: Widget<M>,
    F: for<'d> FnMut(&'d mut Proxy<M, W>, &'d mut SyncContext<M>, Pointer),
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.proxy
    }
}
