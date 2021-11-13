use crate::*;
use std::ops::{Deref, DerefMut};

pub struct Button<W: Geometry + Drawable> {
    widget: W,
    focused: bool,
    cb: Box<dyn for<'d> FnMut(&'d mut W, Pointer) + 'static>,
}

impl<W: Widget> Button<W> {
    pub fn new(
        widget: W,
        cb: impl for<'d> FnMut(&'d mut W, Pointer) + 'static,
    ) -> Self {
        Self {
            widget,
            focused: false,
            cb: Box::new(cb),
        }
    }
}

impl<W: Widget> Geometry for Button<W> {
    fn width(&self) -> f32 {
        self.widget.width()
    }
    fn height(&self) -> f32 {
        self.widget.height()
    }
}

impl<W: Widget> Drawable for Button<W> {
    fn set_color(&mut self, color: u32) {
        self.widget.set_color(color);
    }
    fn draw(&self, ctx: &mut Context, x: f32, y: f32) {
        self.widget.draw(ctx, x, y)
    }
}

impl<W: Widget> Widget for Button<W> {
    fn create_node(&self, x: f32, y: f32) -> RenderNode {
        self.widget.create_node(x, y)
    }
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, ctx: &mut Context, dispatch: &Dispatch) {
        match dispatch {
            Dispatch::Pointer(x, y, pointer) => match pointer {
                Pointer::Leave => {
                    if self.focused {
                        self.focused = false;
                        (self.cb)(&mut self.widget, *pointer)
                    }
                }
                _ => {
                    if *x > wx && *y > wy && *x < wx + self.width() && *y < wy + self.height() {
                        if self.focused {
                            (self.cb)(&mut self.widget, *pointer)
                        } else {
                            self.focused = true;
                            (self.cb)(&mut self.widget, Pointer::Enter)
                        }
                    } else if self.focused {
                        self.focused = false;
                        (self.cb)(&mut self.widget, Pointer::Leave)
                    }
                }
            },
            _ => {}
        }
        self.widget.roundtrip(wx, wy, ctx, dispatch);
    }
}

impl<W: Widget> Deref for Button<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<W: Widget> DerefMut for Button<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}
