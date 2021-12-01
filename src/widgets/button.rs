use crate::*;
use std::ops::{Deref, DerefMut};

pub struct Proxy<W: Widget> {
    child: W,
    damage: bool,
}

impl<W: Widget> Deref for Proxy<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl<W: Widget> DerefMut for Proxy<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.damage = true;
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
            damage: false,
        }
    }
}

impl<W: Widget> Widget for Proxy<W> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        self.child.create_node(x, y)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        self.child.sync(ctx, event);
        if self.damage {
            ctx.request_draw();
            self.damage = false;
        }
    }
}

pub struct Button<W: Widget> {
    focused: bool,
    proxy: Proxy<W>,
    cb: Box<dyn for<'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext, Pointer) + 'static>,
}

impl<W: Widget> Button<W> {
    pub fn new(
        child: W,
        cb: impl for<'d> FnMut(&'d mut Proxy<W>, &'d mut SyncContext, Pointer) + 'static,
    ) -> Self {
        Self {
            proxy: Proxy {
                child,
                damage: false,
            },
            focused: false,
            cb: Box::new(cb),
        }
    }
}

impl<W: Widget> Geometry for Button<W> {
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

impl<W: Widget> Widget for Button<W> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        self.proxy.create_node(x, y)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        if let Event::Pointer(x, y, pointer) = event {
            if x > 0. && y > 0. && x < self.width() && y < self.height() {
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
        self.proxy.sync(ctx, event);
    }
}

impl<W: Widget> Deref for Button<W> {
    type Target = Proxy<W>;
    fn deref(&self) -> &Self::Target {
        &self.proxy
    }
}

impl<W: Widget> DerefMut for Button<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.proxy
    }
}
