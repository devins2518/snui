use crate::*;
use std::ops::{Deref, DerefMut};

pub struct Button<W: Widget> {
    child: W,
    focused: bool,
    cb: Box<dyn for<'d> FnMut(&'d mut W, &'d mut SyncContext, Pointer) + 'static>,
}

impl<W: Widget> Button<W> {
    pub fn new(
        child: W,
        cb: impl for<'d> FnMut(&'d mut W, &'d mut SyncContext, Pointer) + 'static,
    ) -> Self {
        Self {
            child,
            focused: false,
            cb: Box::new(cb),
        }
    }
}

impl<W: Widget> Geometry for Button<W> {
    fn width(&self) -> f32 {
        self.child.width()
    }
    fn height(&self) -> f32 {
        self.child.height()
    }
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        self.child.set_size(width, height)
    }
}

impl<W: Widget> Widget for Button<W> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        self.child.create_node(x, y)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        self.child.sync(ctx, event);
        if let Event::Pointer(x, y, pointer) = event {
            if x > 0.
            && y > 0.
            && x < self.width()
            && y < self.height() {
                if self.focused {
                    (self.cb)(&mut self.child, ctx, pointer);
                } else {
                    self.focused = true;
                    (self.cb)(&mut self.child, ctx, Pointer::Enter);
                }
            } else if self.focused {
                self.focused = false;
                (self.cb)(&mut self.child, ctx, Pointer::Leave);
            }
        } else {
        }
    }
}

impl<W: Widget> Deref for Button<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl<W: Widget> DerefMut for Button<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}
