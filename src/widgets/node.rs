use crate::*;
use std::rc::Rc;
use crate::widgets::*;
use std::ops::{Deref};

#[derive(Clone)]
pub struct Node {
    head: Inner,
    tail: Option<Rc<Self>>,
}

impl Drawable for Node {
    fn set_color(&mut self, color: u32) {
        self.head.set_color(color);
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        let (wwidth, wheight) = (
            self.head.get_width(),
            self.head.get_height(),
        );
        self.head.draw(canvas, width, x, y);
        if let Some(tail) = &self.tail {
            let (dx, dy) = tail.head.get_location(wwidth, wheight).unwrap();
            tail.deref().draw(
                canvas,
                width,
                x + dx,
                y + dy,
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
        let (width, height) = (
            self.head.get_width(),
            self.head.get_height(),
        );
        let ev = self.head.contains(widget_x, widget_y, x, y, event);
        if !ev.is_some() {
            if let Some(tail) = self.tail.as_mut() {
                let (rx, ry) = tail.head.get_location(width, height).unwrap();
                Rc::get_mut(tail).unwrap().contains(widget_x + rx, widget_y + ry, x, y, event)
            } else {
                Damage::None
            }
        } else {
            ev
        }
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.head.resize(width, height)
    }
}

impl Container for Node {
    fn len(&self) -> u32 {
        1 + if let Some(tail) = &self.tail {
            tail.deref().len()
        } else {
            0
        }
    }
    fn add(&mut self, widget: impl Widget + 'static) -> Result<(), Error> {
        if let Some(tail) = &mut self.tail {
            Rc::get_mut(tail).unwrap().add(widget)
        } else {
            self.tail = Some(Rc::new(Node::new(widget)));
            Ok(())
        }
    }
    fn put(&mut self, widget: Inner) -> Result<(), Error> {
        if let Some(tail) = &mut self.tail {
            Rc::get_mut(tail).unwrap().put(widget)
        } else {
            self.tail = Some(Rc::new(Node {
                head: widget,
                tail: None,
            }));
            Ok(())
        }
    }
    fn get_child(&self) -> Result<&dyn Widget, Error> {
        if let Some(widget) = self.tail.as_ref() {
            Ok(&**widget)
        } else {
            Ok(&self.head)
        }
    }
}

impl Transform for Node {
    fn scale(&mut self, f: f32) {
        let e = self.head.resize((self.get_width() as f32 * f) as u32, (self.get_height() as f32 * f) as u32);
        match e {
            Ok(()) => if let Some(tail) = &mut self.tail {
                Rc::get_mut(tail).unwrap().scale(f);
            }
            Err(e) => e.debug()
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
    pub fn new_at(head: impl Widget + 'static, anchor: Anchor, x: u32, y: u32) -> Node {
        Node {
            head: Inner::new_at(head, anchor, x, y),
            tail: None,
        }
    }
    pub fn push(&mut self, node: Node) {
        if let Some(tail) = &mut self.tail {
            Rc::get_mut(tail).unwrap().push(node)
        } else {
            self.tail = Some(Rc::new(node));
        }
    }
    pub fn set_anchor(&mut self, x: u32, y: u32) {
        self.head.set_location(x, y);
    }
    pub fn center(&mut self, widget: impl Widget + 'static) -> Result<(), Error> {
        if let Some(tail) = &mut self.tail {
            Rc::get_mut(tail).unwrap().center(widget)
        } else {
            anchor(self, widget, Anchor::Center, 0, 0)
        }
    }
    pub fn get_location(&self) -> (u32, u32) {
        self.head.get_location(self.head.get_width(), self.head.get_height()).unwrap()
    }
    pub fn set_location(&mut self, x: u32, y: u32) {
        self.head.set_location(x, y);
    }
}

impl Widget for Node {
    fn send_action<'s>(&'s mut self, action: Action, event_loop: &mut Vec<Damage>, widget_x: u32, widget_y: u32) {
        let (width, height) = (
            self.head.get_width(),
            self.head.get_height(),
        );
        let (x, y) = self.head.coords();
        self.head.send_action(action, event_loop, x+widget_x, y+widget_y);
        if let Some(tail) = &mut self.tail {
            let (x, y) = tail.as_ref().head.get_location(width, height).unwrap();
            Rc::get_mut(tail).unwrap().send_action(action, event_loop, x+widget_x, y+widget_y)
        }
    }
}
