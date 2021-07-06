use crate::snui::*;
use crate::widgets::*;
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
            tail.deref().draw(
                canvas,
                x + tail.head.get_location().0,
                y + tail.head.get_location().1,
            );
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
    fn contains(&mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        let msg = self.head.contains(widget_x, widget_y, x, y, event);
        match &msg {
            Damage::None => {
                if let Some(tail) = self.tail.as_mut() {
                    let (rx, ry) = tail.get_location();
                    tail.deref_mut()
                        .contains(widget_x + rx, widget_y + ry, x, y, event)
                } else {
                    Damage::None
                }
            }
            _ => Damage::None,
        }
    }
}

impl Widget for Wbox {}

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
        if let Some(tail) = &mut self.tail {
            tail.deref_mut().add(object)
        } else {
            self.tail = Some(Box::new(Wbox::new(object)));
            Ok(())
        }
    }
    fn get_location(&self) -> (u32, u32) {
        self.head.get_location()
    }
    fn set_location(&mut self, x: u32, y: u32) {
        self.head.set_location(x, y);
    }
    fn put(&mut self, object: Inner) -> Result<(), Error> {
        if let Some(tail) = &mut self.tail {
            tail.deref_mut().put(object)
        } else {
            self.tail = Some(Box::new(Wbox {
                head: object,
                content: Content::Empty,
                tail: None,
            }));
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
            anchor(self, object, Anchor::Center, 0)
        }
    }
    pub fn set_location(&mut self, x: u32, y: u32) {
        self.head.set_location(x, y);
    }
}
