use crate::widgets::*;
use std::ops::{Deref, DerefMut};

pub struct Button<D: Drawable> {
    child: D,
    callback: Box<dyn FnMut(Input)>,
}

impl<D: Drawable> Container for Button<D> {
    fn len(&self) -> u32 { 1 }
    fn add(&mut self, object: impl Drawable + 'static) -> Result<(), Error> {
        Err(Error::Overflow("button", 1))
    }
    fn get_child(&self) -> Vec<&Drawable> {
        vec![&self.child]
    }
}

impl<D: Drawable> Drawable for Button<D> {
    fn set_content(&mut self, content: Content) {}
    fn get_width(&self) -> u32 {
        self.child.get_width()
    }
    fn get_height(&self) -> u32 {
        self.child.get_height()
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
        self.child.draw(canvas, x,y)
    }
    fn contains(&mut self, x: u32, y: u32, event: Input) -> bool {
        if !self.child.contains(x, y, event) {
            (self.callback)(event);
        }
        true
    }
}

impl<D: Drawable> Button<D> {
    pub fn new(child: D, f: impl FnMut(Input) + 'static) -> Button<D> {
        Button {
            child: child,
            callback: Box::new(f)
        }
    }
}
