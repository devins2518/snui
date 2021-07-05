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
    fn get_child(&self) -> Vec<&dyn Drawable> {
        let mut v = Vec::new();
        v.push(self.head.deref());
        if let Some(tail) = &self.tail {
            v.push(tail.as_ref());
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
    pub fn new_at(head: impl Drawable + 'static, x: u32, y: u32) -> Wbox {
        Wbox {
            head: Box::new(head),
            content: Content::Empty,
            pos: (x, y),
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
    pub fn anchor(&mut self, object: impl Drawable + 'static, anchor: Anchor, xoffset: u32, yoffset: u32) -> Result<(), Error> {
        if self.get_width() >= object.get_width() && self.get_height() >= object.get_height()
        {
            let mut x = (self.get_width() - object.get_width()) / 2;
            let mut y = (self.get_height() - object.get_height()) / 2;
            match anchor {
                Anchor::Left => x = xoffset,
                Anchor::Right => x = self.get_width() - object.get_width() - xoffset,
                Anchor::Top => y = yoffset,
                Anchor::Bottom => y = self.get_height() - object.get_height() - yoffset,
                Anchor::Center => {}
                Anchor::TopRight => {
                    x = self.get_width() - object.get_width() - xoffset;
                    y = self.get_height() - object.get_height() - yoffset;
                }
                Anchor::TopLeft => {
                    x = xoffset;
                    y = self.get_height() - object.get_height() - yoffset;
                }
                Anchor::BottomRight => {
                    x = self.get_width() - object.get_width() - xoffset;
                    y = yoffset;
                }
                Anchor::BottomLeft => {
                    x = xoffset;
                    y = yoffset;
                }
            }
            self.push(Wbox::new_at(object, x, y));
            Ok(())
        } else {
            Err(Error::Dimension("wbox",object.get_width(),object.get_height()))
        }
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
    pub fn center(&mut self, object: impl Drawable + 'static) -> Result<(), Error> {
        if let Some(tail) = &mut self.tail {
            tail.deref_mut().center(object)
        } else {
            if object.get_width() < self.get_width()
        	&& object.get_height() < self.get_height() {
                let x = (self.head.get_width() - object.get_width())/2;
                let y = (self.head.get_height() - object.get_height())/2;
                self.tail = Some(Box::new(Wbox::new_at(object, x, y)));
                Ok(())
            } else {
                self.add(object)
            }
        }
    }
}
