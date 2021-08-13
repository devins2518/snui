use crate::*;
use std::rc::Rc;
use crate::widgets::*;
use std::cell::RefCell;
use crate::widgets::active::pointer;
use crate::widgets::active::command::Command;

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
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.widget.resize(width, height)
    }
    fn contains<'d>(
        &'d mut self,
        widget_x: u32,
        widget_y: u32,
        x: u32,
        y: u32,
        event: pointer::Event,
    ) -> Damage {
        self.widget
            .contains(widget_x + self.size.0, widget_y + self.size.3, x, y, event)
    }
}

impl<W: Widget + Clone> Drawable for Border<W> {
    fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        let bwidth = self.get_width();
        let bheight = self.get_height();

        Rectangle::new(bwidth, self.size.0, self.color).draw(canvas, width, x, y);
        Rectangle::new(bwidth, self.size.2, self.color).draw(
            canvas,
            width,
            x,
            y + bheight - self.size.2,
        );
        Rectangle::new(self.size.1, bheight, self.color).draw(
            canvas,
            width,
            x + bwidth - self.size.1,
            y,
        );
        Rectangle::new(self.size.3, bheight, self.color).draw(canvas, width, x, y);

        self.widget
            .draw(canvas, width, x + self.size.0, y + self.size.3);
    }
}

impl<W: Widget + Clone> Widget for Border<W> {
 	fn send_command<'s>(&'s mut self, command: Command, damages: &mut Vec<Damage<'s>>, x: u32, y: u32) {
        self.widget
            .send_command(command, damages, x + self.size.0, y + self.size.3);
    }
}

impl<W: Widget + Clone> Border<W> {
    pub fn new(widget: W, size: u32, color: u32) -> Self {
        Self {
            widget,
            color,
            size: (size, size, size, size),
        }
    }
    pub fn set_border_size(&mut self, top: u32, right: u32, bottom: u32, left: u32) {
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
        self.widget.get_width() + self.padding.1 + self.padding.3
    }
    fn get_height(&self) -> u32 {
        self.widget.get_height() + self.padding.0 + self.padding.2
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.widget.resize(width, height)
    }
    fn contains<'d>(
        &'d mut self,
        widget_x: u32,
        widget_y: u32,
        x: u32,
        y: u32,
        event: pointer::Event,
    ) -> Damage {
        self.widget.contains(
            widget_x + self.padding.3,
            widget_y + self.padding.0,
            x,
            y,
            event,
        )
    }
}

impl<B: Widget, W: Widget + Clone> Drawable for Background<B, W> {
    fn set_color(&mut self, color: u32) {
        self.background.as_ref().borrow_mut().set_color(color);
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        let bwidth = self.get_width();
        let bheight = self.get_height();

        self.background
            .as_ref()
            .borrow_mut()
            .resize(bwidth, bheight)
            .unwrap();
        self.background.as_ref().borrow().draw(canvas, width, x, y);

        self.widget
            .draw(canvas, width, x + self.padding.3, y + self.padding.0);
    }
}

impl<B: Widget, W: Widget + Clone> Widget for Background<B, W> {
 	fn send_command<'s>(&'s mut self, command: Command, damages: &mut Vec<Damage<'s>>, x: u32, y: u32) {
        // self.background.as_ref().borrow_mut().send_command(command, damages, x, y);
        self.widget
            .send_command(command, damages, x + self.padding.0, y + self.padding.3)
    }
}

impl<B: Widget, W: Widget + Clone> Background<B, W> {
    pub fn new(widget: W, background: B, padding: u32) -> Self {
        Self {
            widget: widget,
            background: Rc::new(RefCell::new(background)),
            padding: (padding, padding, padding, padding),
        }
    }
    pub fn solid(widget: W, color: u32, padding: u32) -> Background<Rectangle, W> {
        Background {
            widget: widget,
            background: Rc::new(RefCell::new(Rectangle::new(0, 0, color))),
            padding: (padding, padding, padding, padding),
        }
    }
    pub fn set_padding(&mut self, top: u32, right: u32, bottom: u32, left: u32) {
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
        _event: pointer::Event,
    ) -> Damage {
        Damage::None
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
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

        if self.color != 0 {
            let mut index = ((x + (y * width as u32)) * 4) as usize;
            for _ in 0..self.height {
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
}

impl Widget for Rectangle {
 	fn send_command<'s>(&'s mut self, _command: Command, _damages: &mut Vec<Damage>, _x: u32, _y: u32) {}
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
pub struct Revealer<N: Widget, R: Widget> {
    state: bool,
    normal: N,
    reveal: R,
}

impl<N: Widget, R: Widget> Drawable for Revealer<N, R> {
    fn set_color(&mut self, color: u32) {
        if self.state {
            self.reveal.set_color(color);
        } else {
            self.normal.set_color(color);
        }
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        if self.state {
            self.reveal.draw(canvas, width, x, y)
        } else {
            self.normal.draw(canvas, width, x, y)
        }
    }
}

impl<N: Widget, R: Widget> Geometry for Revealer<N, R> {
    fn get_width(&self) -> u32 {
        if self.state {
            self.reveal.get_width()
        } else {
            self.normal.get_width()
        }
    }
    fn get_height(&self) -> u32 {
        if self.state {
            self.reveal.get_height()
        } else {
            self.normal.get_height()
        }
    }
    fn contains<'d>(
        &'d mut self,
        widget_x: u32,
        widget_y: u32,
        x: u32,
        y: u32,
        event: pointer::Event,
    ) -> Damage {
        if self.state {
            self.reveal.contains(widget_x, widget_y, x, y, event)
        } else {
            self.normal.contains(widget_x, widget_y, x, y, event)
        }
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        if self.state {
            self.reveal.resize(width, height)
        } else {
            self.normal.resize(width, height)
        }
    }
}

impl<N: Widget, R: Widget> Revealer<N, R> {
    pub fn new(normal: N, reveal: R) -> Revealer<N, R> {
        Revealer {
            state: false,
            normal,
            reveal,
        }
    }
    pub fn toggle(&mut self) {
        if self.state {
            self.state = false
        } else {
            self.state = true
        }
    }
}

impl<N: Widget, R: Widget> Widget for Revealer<N, R> {
 	fn send_command<'s>(&'s mut self, command: Command, damages: &mut Vec<Damage<'s>>, x: u32, y: u32) {
        if self.state {
            self.reveal.send_command(command, damages, x, y)
        } else {
            self.normal.send_command(command, damages, x, y)
        }
    }
}
