use crate::snui::*;
use crate::widgets::Surface;
use std::ops::{Deref, DerefMut};

// that can unfold it's inner content
pub struct Wbox {
    head: Box<Drawable>,
    content: Content,
    pos: (u32, u32),
    tail: Option<Box<Self>>,
}

impl Drawable for Wbox {
    fn get_width(&self) -> u32 {
        self.head.get_width()
    }
    fn get_height(&self) -> u32 {
        self.head.get_height()
    }
    fn set_content(&mut self, content: Content) {
        self.content = content;
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
        self.head.draw(canvas, x, y);
        if let Some(tail) = &self.tail {
            tail.deref().draw(canvas, x+tail.pos.0, y+tail.pos.1);
        }
    }
    fn contains(&mut self, x: u32, y: u32, event: Input) -> bool {
        if !self.head.contains(x, y, event) {
            if let Some(tail) = &mut self.tail {
                tail.deref_mut().contains(x, y, event)
            } else {
                false
            };
        }
        false
    }
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
    fn add(&mut self, object: impl Drawable + 'static) -> Result<(), Error> {
        if let Some(tail) = &mut self.tail {
            tail.deref_mut().add(object)
        } else {
            self.tail = Some(Box::new(Wbox::new(object)));
            Ok(())
        }
    }
    // Returns the list of child geometries
    fn get_child(&self) -> Vec<&dyn Drawable> {
        let mut v = Vec::new();
        v.push(self.head.deref());
        if let Some(tail) = &self.tail {
            let mut t = tail.as_ref().get_child();
            v.append(&mut t);
        }
        v
    }
}

impl Wbox {
    pub fn new(head: impl Drawable + 'static) -> Wbox {
        Wbox {
            head: Box::new(head),
            content: Content::Empty,
            pos: (0, 0),
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
        self.pos = (x, y);
    }
    pub fn to_surface(&self) -> Surface {
        let mut surface = Surface::new(
            self.get_width(),
            self.get_height(),
            Content::Empty,
        );
        self.draw(&mut surface, 0, 0);
        surface
    }
    pub fn center(&self, canvas: &mut Surface, x: u32, y: u32) {
        self.head.draw(canvas, x, y);
        if let Some(tail) = &self.tail {
            let x = (self.head.get_width() - tail.head.get_width())/2;
            let y = (self.head.get_height() - tail.head.get_height())/2;
            tail.deref().draw(canvas, x, y);
        }
    }
}
