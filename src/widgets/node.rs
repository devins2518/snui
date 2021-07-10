use crate::snui::*;
use crate::widgets::*;
use std::ops::{Deref, DerefMut};

// that can unfold it's inner content
pub struct Node {
    head: Inner,
    tail: Option<Box<Self>>,
}

impl Drawable for Node {
    fn set_content(&mut self, content: Content) {
        self.head.set_content(content);
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

impl Geometry for Node {
    fn get_width(&self) -> u32 {
        self.head.get_width()
    }
    fn get_height(&self) -> u32 {
        self.head.get_height()
    }
    fn contains(&mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        // let (fx, fy) = self.head.get_location();
        let msg = self.head.contains(widget_x, widget_y, x, y, event);
        match &msg {
            Damage::None => {
                if let Some(tail) = self.tail.as_mut() {
                    let (rx, ry) = tail.get_location();
                    tail.contains(widget_x + rx, widget_y + ry, x, y, event)
                } else {
                    Damage::None
                }
            }
            _ => msg,
        }
    }
}

impl Widget for Node {}

impl Container for Node {
    fn len(&self) -> u32 {
        1 + if let Some(tail) = &self.tail {
            tail.deref().len()
        } else {
            0
        }
    }
    // Appends an widget at the end of a Container
    fn add(&mut self, widget: impl Widget + 'static) -> Result<(), Error> {
        if let Some(tail) = &mut self.tail {
            tail.deref_mut().add(widget)
        } else {
            self.tail = Some(Box::new(Node::new(widget)));
            Ok(())
        }
    }
    fn put(&mut self, widget: Inner) -> Result<(), Error> {
        if let Some(tail) = &mut self.tail {
            tail.deref_mut().put(widget)
        } else {
            self.tail = Some(Box::new(Node {
                head: widget,
                tail: None,
            }));
            Ok(())
        }
    }
    fn get_child(&self) -> Result<&dyn Widget,Error> {
        if let Some(widget) = self.tail.as_ref() {
            Ok(&**widget)
        } else {
            Ok(&self.head)
        }
    }
}

impl Node {
    pub fn new(head: impl Widget + 'static) -> Node {
        Node {
            head: Inner::new(head),
            tail: None,
        }
    }
    pub fn new_at(head: impl Widget + 'static, x: u32, y: u32) -> Node {
        Node {
            head: Inner::new_at(head, x, y),
            tail: None,
        }
    }
    pub fn push(&mut self, node: Node) {
        if let Some(tail) = &mut self.tail {
            tail.deref_mut().push(node)
        } else {
            self.tail = Some(Box::new(node));
        }
    }
    pub fn set_anchor(&mut self, x: u32, y: u32) {
        self.head.set_location(x, y);
    }
    pub fn center(&mut self, widget: impl Widget + 'static) -> Result<(), Error> {
        if let Some(tail) = &mut self.tail {
            tail.deref_mut().center(widget)
        } else {
            anchor(self, widget, Anchor::Center, 0)
        }
    }
    pub fn get_location(&self) -> (u32, u32) {
        self.head.get_location()
    }
    pub fn set_location(&mut self, x: u32, y: u32) {
        self.head.set_location(x, y);
    }
}
