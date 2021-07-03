use crate::widgets::*;
use std::ops::{Deref, DerefMut};

pub struct Button {
    child: Box<dyn Drawable>,
    callback: Box<dyn FnMut(Input)>,
}

impl Container for Button {
    fn len(&self) -> u32 { 1 }
    fn add(&mut self, object: impl Drawable + 'static) -> Result<(), Error> {
        Err(Error::Overflow("button", 1))
    }
    fn get_child(&self) -> Vec<&Drawable> {
        vec![&*self.child]
    }
}

impl Drawable for Button {
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
        if !self.child.deref_mut().contains(x, y, event) {
            (self.callback)(event);
        }
        true
    }
}

impl Button {
    pub fn new(child: impl Drawable + 'static, f: impl FnMut(Input) + 'static) -> Button {
        Button {
            child: Box::new(child),
            callback: Box::new(f)
        }
    }
}
