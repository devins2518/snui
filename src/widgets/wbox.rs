use crate::snui::*;
use crate::widgets::{Surface, Inner};
use std::ops::{Deref, DerefMut};

// that can unfold it's inner content
pub struct Wbox {
    head: Inner,
    content: Content,
    tail: Option<Box<Self>>,
}

impl Drawable for Wbox {
    fn set_content(&mut self, content: Content) {
        self.content = content;
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
        self.head.draw(canvas, x, y);
        if let Some(tail) = &self.tail {
            tail.deref().draw(canvas, x + tail.head.get_location().0, y + tail.head.get_location().1);
        }
    }
}

impl Geometry for Wbox {
    fn get_width(&self) -> u32 {
        self.head.get_width()
    }
    fn get_height(&self) -> u32 {
        self.head.get_height()
    }
    fn get_location(&self) -> (u32, u32) {
        self.head.get_location()
    }
    fn set_location(&mut self, x: u32, y: u32) {
        // let (hx, hy) = self.get_location();
        self.head.set_location(x, y);
    }
    fn contains(&mut self, x: u32, y: u32, event: Input) -> Damage {
        let msg = self.head.contains(x, y, event);
        match &msg {
            Damage::None => {
                if let Some(tail) = &mut self.tail {
                    tail.deref_mut().contains(x, y, event)
                } else {
                    Damage::None
                }
            }
            _ => Damage::None
        }
    }
}

impl Widget for Wbox {
}

impl Container for Wbox {
    fn len(&self) -> u32 {
        1 + if let Some(tail) = &self.tail {
            tail.deref().len()
        } else {
            0
        }
    }
    // Appends an object at the end of a Container
    fn add(&mut self, object: impl Widget + 'static) -> Result<(), Error> {
        let (x, y) = self.head.get_location();
        if let Some(tail) = &mut self.tail {
            tail.deref_mut().add(object)
        } else {
            self.tail = Some(Box::new(Wbox::new_at(object, x, y)));
            Ok(())
        }
    }
    /*
    fn get_child(&self) -> Vec<&Inner> {
        let mut v = Vec::new();
        v.push(&self.head);
        if let Some(tail) = &self.tail {
            v.push(&tail.as_ref().head);
        }
        v
    }
    */
}


impl Wbox {
    pub fn new(head: impl Widget + 'static) -> Wbox {
        Wbox {
            head: Inner::new(head),
            content: Content::Empty,
            tail: None,
        }
    }
    pub fn new_at(head: impl Widget + 'static, x: u32, y: u32) -> Wbox {
        Wbox {
            head: Inner::new_at(head, x, y),
            content: Content::Empty,
            tail: None,
        }
    }
    pub fn push(&mut self, node: Wbox) {
        if let Some(tail) = &mut self.tail {
            tail.deref_mut().push(node)
        } else {
            self.tail = Some(Box::new(node));
        }
    }
    pub fn set_anchor(&mut self, x: u32, y: u32) {
        self.head.set_location(x, y);
    }
    pub fn center(&mut self, object: impl Widget + 'static) -> Result<(), Error> {
        if let Some(tail) = &mut self.tail {
            tail.deref_mut().center(object)
        } else {
            if object.get_width() < self.get_width() && object.get_height() < self.get_height() {
                let x = (self.head.get_width() - object.get_width()) / 2;
                let y = (self.head.get_height() - object.get_height()) / 2;
                self.tail = Some(Box::new(Wbox::new_at(object, x, y)));
                Ok(())
            } else {
                self.add(object)
            }
        }
    }
    pub fn set_location(&mut self, x: u32, y: u32) {
        self.head.set_location(x, y);
    }
}
