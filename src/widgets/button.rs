use crate::*;
use std::ops::{Deref, DerefMut};

pub struct Proxy<W: Widget> {
    child: W,
    damage: Damage,
    queue_draw: bool,
}

impl<W: Widget> Deref for Proxy<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl<W: Widget> DerefMut for Proxy<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.damage = self.damage.max(Damage::Some);
        &mut self.child
    }
}

impl<W: Widget> Geometry for Proxy<W> {
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

impl<W: Widget> Proxy<W> {
    pub fn new(child: W) -> Self {
        Proxy {
            child,
            queue_draw: true,
            damage: Damage::Some,
        }
    }
}

impl<W: Widget> Widget for Proxy<W> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        if self.queue_draw || self.damage.is_some() {
            self.damage = Damage::None;
            return self.child.create_node(x, y);
        }
        RenderNode::None
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) -> Damage {
        self.damage = self.damage.max(self.child.sync(ctx, event));
        self.queue_draw = self.damage.is_some() || event == Event::Frame;
        self.damage
    }
}

pub struct Button<W, F>
where
    W: Widget,
    F: for <'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext, Pointer)
{
    focused: bool,
    proxy: Proxy<W>,
    cb: F
}

impl<W, F> Button<W, F>
where
    W: Widget,
    F: for <'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext, Pointer)
{
    pub fn new(
        child: W,
        cb: F
    ) -> Self {
        Self {
            proxy: Proxy::new(child),
            focused: false,
            cb,
        }
    }
}

impl<W, F> Geometry for Button<W, F>
where
    W: Widget,
    F: for <'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext, Pointer)
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

impl<W, F> Widget for Button<W, F>
where
    W: Widget,
    F: for <'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext, Pointer)
{
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        self.proxy.create_node(x, y)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) -> Damage {
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

impl<W, F> Deref for Button<W, F>
where
    W: Widget,
    F: for <'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext, Pointer)
{
    type Target = Proxy<W>;
    fn deref(&self) -> &Self::Target {
        &self.proxy
    }
}

impl<W, F> DerefMut for Button<W, F>
where
    W: Widget,
    F: for <'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext, Pointer)
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.proxy
    }
}
