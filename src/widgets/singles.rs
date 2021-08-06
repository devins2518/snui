use crate::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::widgets::*;

#[derive(Clone)]
pub struct Border<W: Widget + Clone> {
    pub widget: W,
    color: u32,
    size: (u32, u32, u32, u32),
}

impl<W: Widget + Clone> Geometry for Border<W> {
    fn get_width(&self) -> u32 {
        self.widget.get_width() + self.size.0 + self.size.2
    }
    fn get_height(&self) -> u32 {
        self.widget.get_height() + self.size.1 + self.size.3
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(),Error> {
        self.widget.resize(width, height)
    }
    fn contains<'d>(&'d mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        self.widget.contains(widget_x + self.size.0, widget_y + self.size.3, x, y, event)
    }
}

impl<W: Widget + Clone> Drawable for Border<W>{
    fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        let bwidth = self.get_width();
        let bheight = self.get_height();

		Rectangle::new(bwidth, self.size.0, self.color).draw(canvas, width, x, y);
		Rectangle::new(bwidth, self.size.2, self.color).draw(canvas, width, x, y + bheight - self.size.2);
		Rectangle::new(self.size.1, bheight, self.color).draw(canvas, width, x + bwidth - self.size.1, y);
		Rectangle::new(self.size.3, bheight, self.color).draw(canvas, width, x, y);

		self.widget.draw(canvas, width, x + self.size.0, y + self.size.3);
    }
}

impl<W: Widget + Clone> Widget for Border<W>{
    fn send_action<'s>(&'s mut self, action: Action) {
        self.widget.send_action(action);
    }
}

impl<W: Widget + Clone> Border<W>{
    pub fn new(widget: W, size: u32, color: u32) -> Self {
        Self {
            widget,
            color,
            size: (size, size, size, size)
        }
    }
    fn set_border_size(&mut self, top: u32, right: u32, bottom: u32, left: u32) {
        self.size = (top, right, bottom, left);
    }
}

#[derive(Clone)]
pub struct Background<B: Widget, W: Widget + Clone> {
    pub widget: W,
    pub background: Rc<RefCell<B>>,
    padding: (u32, u32, u32, u32),
}

impl<B: Widget, W: Widget + Clone> Geometry for Background<B, W> {
    fn get_width(&self) -> u32 {
        self.widget.get_width() + self.padding.0 + self.padding.2
    }
    fn get_height(&self) -> u32 {
        self.widget.get_height() + self.padding.1 + self.padding.3
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(),Error> {
        self.widget.resize(width, height)
    }
    fn contains<'d>(&'d mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        self.widget.contains(widget_x + self.padding.0, widget_y + self.padding.3, x, y, event)
    }
}

impl<B: Widget, W: Widget + Clone> Drawable for Background<B, W>{
    fn set_color(&mut self, color: u32) {
        self.background.as_ref().borrow_mut().set_color(color);
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        let bwidth = self.get_width();
        let bheight = self.get_height();

        self.background.as_ref().borrow_mut().resize(bwidth, bheight).unwrap();
        self.background.as_ref().borrow().draw(canvas, width, x, y);

		self.widget.draw(canvas, width, x + self.padding.0, y + self.padding.3);
    }
}

impl<B: Widget, W: Widget + Clone> Widget for Background<B, W>{
    fn send_action<'s>(&'s mut self, action: Action) {
        self.widget.send_action(action);
        self.background.as_ref().borrow_mut().send_action(action);
    }
}

impl<B: Widget, W: Widget + Clone> Background<B, W>{
    pub fn new(widget: W, background: B, padding: u32) -> Self {
        Self {
            widget: widget,
            background: Rc::new(RefCell::new(background)),
            padding: (padding, padding, padding, padding),
        }
    }
    pub fn solid(widget: W, color: u32, padding: u32) -> Background<Rectangle, W>{
        Background {
            widget: widget,
            background: Rc::new(RefCell::new(Rectangle::new(0, 0, color))),
            padding: (padding, padding, padding, padding),
        }
    }
    fn set_padding(&mut self, top: u32, right: u32, bottom: u32, left: u32) {
        self.padding = (top, right, bottom, left);
    }
}

/*
* The most basic widget one can create. It's the basis of everything else.
*/
#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    width: u32,
    height: u32,
    radius: u32,
    color: u32,
}

impl Geometry for Rectangle {
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        self.height
    }
    fn contains(
        &mut self,
        _widget_x: u32,
        _widget_y: u32,
        _x: u32,
        _y: u32,
        _event: Input,
    ) -> Damage {
        Damage::None
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(),Error> {
        self.width = width;
        self.height = height;
        Ok(())
    }
}

impl Drawable for Rectangle {
    fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        let buf = self.color.to_ne_bytes();

        let mut index = ((x + (y * width as u32)) * 4) as usize;
        for _ in 0.. self.height {
            if index >= canvas.len() {
                break;
            } else {
                let mut writer = &mut canvas[index..];
                for _ in 0..self.width {
                    writer.write_all(&buf).unwrap();
                }
                writer.flush().unwrap();
                index += width as usize * 4;
            }
        }
    }
}

impl Widget for Rectangle {
    fn send_action<'s>(&'s mut self, _action: Action) {}
}

impl Rectangle {
    pub fn new(width: u32, height: u32, color: u32) -> Rectangle {
        Rectangle {
            color,
            width,
            height,
            radius: 0,
        }
    }
    pub fn empty(width: u32, height: u32) -> Rectangle {
        Rectangle {
            color: 0,
            width,
            height,
            radius: 0,
        }
    }
    pub fn square(size: u32, color: u32) -> Rectangle {
        Rectangle {
            color,
            width: size,
            height: size,
            radius: 0,
        }
    }
    pub fn set_radius(&mut self, radius: u32) {
        self.radius = radius;
    }
}

#[derive(Clone)]
pub struct Inner {
    x: u32,
    y: u32,
    mapped: bool,
    entered: bool,
    anchor: Anchor,
    widget: Rc<dyn Widget>,
}

impl Geometry for Inner {
    fn get_width(&self) -> u32 {
        self.widget.as_ref().get_width()
    }
    fn get_height(&self) -> u32 {
        self.widget.as_ref().get_height()
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(),Error> {
        Rc::get_mut(&mut self.widget).unwrap().resize(width, height)
    }
    fn contains<'d>(&'d mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage {
        if self.entered
        	&& x < widget_x + 10
        	&& y < widget_x + 10
        	&& x < widget_x + self.get_width() - 10
        	&& y < widget_y + self.get_height() - 10
    	{
            self.entered = false;
            Rc::get_mut(&mut self.widget).unwrap().contains(widget_x, widget_y, x, y, event)
        } else if x > widget_x
            && y > widget_y
            && x < widget_x + self.get_width()
            && y < widget_y + self.get_height()
        {
            self.entered = true;
            // println!("{} {}", x, y);
            Rc::get_mut(&mut self.widget).unwrap().contains(widget_x, widget_y, x, y, event)
        } else {
            Damage::None
        }
    }
}

impl Container for Inner {
    fn len(&self) -> u32 {
        1
    }
    fn add(&mut self, _widget: impl Drawable + 'static) -> Result<(), Error> {
        Err(Error::Overflow("inner", 1))
    }
    fn put(&mut self, _widget: Inner) -> Result<(), Error> {
        Err(Error::Overflow("inner", 1))
    }
    fn get_child(&self) -> Result<&dyn Widget, Error> {
        Ok(self.widget.as_ref())
    }
}

impl Drawable for Inner {
    fn set_color(&mut self, color: u32) {
        Rc::get_mut(&mut self.widget).unwrap().set_color(color)
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.widget.as_ref().draw(canvas, width, x, y);
    }
}

impl Widget for Inner {
    fn send_action<'s>(&'s mut self, action: Action) {
        Rc::get_mut(&mut self.widget).unwrap().send_action(action);
    }
}

impl Inner {
    pub fn new(widget: impl Widget + 'static) -> Inner {
        Inner {
            x: 0,
            y: 0,
            mapped: false,
            entered: false,
            anchor: Anchor::TopLeft,
            widget: Rc::new(widget),
        }
    }
    pub fn new_at(widget: impl Widget + 'static, anchor: Anchor, x:  u32, y: u32) -> Inner {
        Inner {
            x,
            y,
            anchor,
            mapped: false,
            entered: false,
            widget: Rc::new(widget),
        }
    }
    pub fn get_anchor(&self) -> Anchor {
        self.anchor
    }
    pub fn is_mapped(&self) -> bool {
        self.mapped
    }
    pub fn map(&mut self) {
        self.mapped = true;
    }
    pub fn unmap(&mut self) {
        self.mapped = false;
    }
    pub fn coords(&self) -> (u32, u32) {
        (self.x, self.y)
    }
    pub fn get_location(&self, width: u32, height: u32) -> Result<(u32, u32), Error> {
        Ok(match self.anchor {
            Anchor::Left => (
                self.x,
                (height - self.get_height() + self.y)/2
            ),
            Anchor::Right => (
                width - self.get_width() - self.x,
                (height - self.get_height() + self.y)/2
            ),
            Anchor::Top => ((width - self.get_width() + self.x)/2, self.y),
            Anchor::Bottom => ((width - self.get_width() + self.x)/2, height - self.y - self.get_height()),
            Anchor::Center => (
                if width >= self.get_width() {
                    (width - self.get_width() + self.x)/2
                } else { 0 },
                if height >= self.get_height() {
                    (height - self.get_height() + self.y)/2
                } else { 0 },
            ),
            Anchor::TopRight => (width - self.x - self.get_width(), self.y),
            Anchor::TopLeft => (self.x, self.y),
            Anchor::BottomRight => (
                width - self.x - self.get_width(),
                height - self.y - self.get_height()
            ),
            Anchor::BottomLeft => (
                self.x,
                height - self.y - self.get_height()
            )
        })
    }
    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
    }
    pub fn set_location(&mut self, x: u32, y: u32) {
        self.x = x;
        self.y = y;
    }
    pub fn translate(&mut self, x: u32, y: u32) {
        self.x += x;
        self.y += y;
    }
}
