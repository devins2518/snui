use crate::*;
use crate::widgets::Rectangle;

pub struct Wbox {
    width: u32,
    height: u32,
    pub widgets: Vec<Inner>,
    background: u32,
}

pub struct Inner {
    x: u32,
    y: u32,
    mapped: bool,
    // entered: bool,
    anchor: Anchor,
    widget: Box<dyn Widget>,
}

impl Geometry for Inner {
    fn get_width(&self) -> u32 {
        self.widget.as_ref().get_width()
    }
    fn get_height(&self) -> u32 {
        self.widget.as_ref().get_height()
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
        event: Event,
    ) -> Damage {
        self.widget.contains(widget_x, widget_y, x, y, event)
    }
}

impl Container for Inner {
    fn len(&self) -> usize {
        1
    }
    fn add(&mut self, _widget: impl Drawable + 'static) -> Result<(), Error> {
        Err(Error::Overflow("inner", 1))
    }
    fn get_child(&self) -> Result<&dyn Widget, Error> {
        Ok(self.widget.as_ref())
    }
}

impl Drawable for Inner {
    fn set_color(&mut self, color: u32) {
        self.widget.set_color(color)
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.widget.draw(canvas, width, x, y);
    }
}

impl Widget for Inner {
    fn send_command<'s>(
        &'s mut self,
        command: Command,
        damage_queue: &mut Vec<Damage<'s>>,
        x: u32,
        y: u32,
    ) {
        self.widget.send_command(command, damage_queue, x, y);
    }
}

impl Inner {
    pub fn new(widget: impl Widget + 'static) -> Inner {
        Inner {
            x: 0,
            y: 0,
            mapped: true,
            // entered: false,
            anchor: Anchor::TopLeft,
            widget: Box::new(widget),
        }
    }
    pub fn new_at(widget: impl Widget + 'static, anchor: Anchor, x: u32, y: u32) -> Inner {
        Inner {
            x,
            y,
            anchor,
            mapped: true,
            // entered: false,
            widget: Box::new(widget),
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
        let widget_width = self.get_width();
        let widget_height = self.get_height();
        match self.anchor {
            Anchor::Left => {
                if height >= widget_height {
                    return Ok((self.x, (height - widget_height + self.y) / 2));
                }
            }
            Anchor::Right => {
                if height >= widget_height && width >= widget_height {
                    return Ok((
                        width - widget_width - self.x,
                        (height - widget_height + self.y) / 2,
                    ));
                }
            }
            Anchor::Top => {
                if width >= widget_width {
                    return Ok(((width - widget_width + self.x) / 2, self.y));
                }
            }
            Anchor::Bottom => {
                if height > self.y + widget_height {
                    return Ok((
                        (width - widget_width + self.x) / 2,
                        height - self.y - widget_height,
                    ));
                }
            }
            Anchor::Center => {
                return Ok((
                    if width >= widget_width {
                        (width - widget_width + self.x) / 2
                    } else {
                        0
                    },
                    if height >= widget_height {
                        (height - widget_height + self.y) / 2
                    } else {
                        0
                    },
                ))
            }
            Anchor::TopRight => {
                if width > self.x + widget_width {
                    return Ok((width - self.x - widget_width, self.y));
                }
            }
            Anchor::TopLeft => return Ok((self.x, self.y)),
            Anchor::BottomRight => {
                if width > self.x + widget_width && height > self.y + widget_height {
                    return Ok((
                        width - self.x - widget_width,
                        height - self.y - widget_height,
                    ));
                }
            }
            Anchor::BottomLeft => {
                if height > self.y + widget_height {
                    return Ok((self.x, height - self.y - widget_height));
                }
            }
        }
        Err(Error::Dimension("wbox", widget_width, widget_height))
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

impl Geometry for Wbox {
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        self.height
    }
    fn contains<'d>(
        &'d mut self,
        widget_x: u32,
        widget_y: u32,
        x: u32,
        y: u32,
        event: Event,
    ) -> Damage {
        let width = self.get_width();
        let height = self.get_height();
        for l in &mut self.widgets {
            let (dx, dy) = l.get_location(width, height).unwrap();
            let ev = l.contains(widget_x + dx, widget_y + dy, x, y, event);
            if ev.is_some() {
                return ev;
            }
        }
        Damage::None
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.width = width;
        self.height = height;
        Ok(())
    }
}

impl Drawable for Wbox {
    fn set_color(&mut self, color: u32) {
        self.background = color;
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        let sw = self.get_width();
        let sh = self.get_height();
        if self.background != 0 {
            let rectangle = Rectangle::new(sw, sh, self.background);
            rectangle.draw(canvas, width, x, y);
        }
        for w in &self.widgets {
            match w.get_location(sw, sh) {
                Ok((dx, dy)) => {
                    if w.is_mapped() && dx <= sw && dy <= sh {
                        w.draw(canvas, width, x + dx, y + dy)
                    }
                }
                Err(e) => e.debug(),
            }
        }
    }
}

impl Container for Wbox {
    fn len(&self) -> usize {
        self.widgets.len()
    }
    fn add(&mut self, _widget: impl Widget + 'static) -> Result<(), Error> {
        Err(Error::Message("get_child is not valid on \"wbox\""))
    }
    fn get_child(&self) -> Result<&dyn Widget, Error> {
        Err(Error::Message("get_child is not valid on \"wbox\""))
    }
}

impl Wbox {
    pub fn new(width: u32, height: u32) -> Self {
        Wbox {
            width,
            height,
            background: 0,
            widgets: Vec::new(),
        }
    }

    pub fn anchor(&mut self, widget: impl Widget + 'static, anchor: Anchor, x: u32, y: u32) {
        self.widgets.push(Inner::new_at(widget, anchor, x, y));
    }

    pub fn unmap(&mut self, i: usize) {
        if i < self.widgets.len() {
            self.widgets[i].unmap();
        }
    }
    pub fn map(&mut self, i: usize) {
        if i < self.widgets.len() {
            self.widgets[i].map();
        }
    }
    pub fn map_all(&mut self) {
        for w in &mut self.widgets {
            w.map();
        }
    }
}

impl Widget for Wbox {
    fn send_command<'s>(
        &'s mut self,
        command: Command,
        damage_queue: &mut Vec<Damage<'s>>,
        x: u32,
        y: u32,
    ) {
        let width = self.get_width();
        let height = self.get_height();
        for w in &mut self.widgets {
            let (dx, dy) = w.get_location(width, height).unwrap();
            w.send_command(command, damage_queue, x + dx, y + dy);
        }
    }
}
