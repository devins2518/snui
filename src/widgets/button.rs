use crate::widgets::*;

pub struct Button<W: Widget> {
    widget: W,
    callback: Box<dyn FnMut(&mut W, Input) -> Damage>,
}

impl<W: Widget> Container for Button<W> {
    fn len(&self) -> u32 {
        1
    }
    fn add(&mut self, _object: impl Widget + 'static) -> Result<(), Error> {
        Err(Error::Overflow("button", 1))
    }
    /*
    fn get_child(&self) -> Vec<&Inner> {
        vec![&self.widget]
    }
    */
}

impl<W: Widget> Geometry for Button<W> {
    fn get_width(&self) -> u32 {
        self.widget.get_width()
    }
    fn get_height(&self) -> u32 {
        self.widget.get_height()
    }
    fn get_location(&self) -> (u32, u32) {
        self.widget.get_location()
    }
    fn set_location(&mut self, x: u32, y: u32) {
        self.widget.set_location(x, y);
    }
    fn contains(&mut self, x: u32, y: u32, event: Input) -> Damage {
        let (sx, sy) = self.widget.get_location();
        if x > sx
            && y > sy
            && x < sx + self.widget.get_width()
            && y < sy + self.widget.get_height()
        {
            (self.callback)(&mut self.widget, event)
        } else {
            Damage::None
        }
    }
}

impl<W: Widget> Drawable for Button<W> {
    fn set_content(&mut self, content: Content) {
        self.widget.set_content(content);
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
        self.widget.draw(canvas, x, y)
    }
}

impl<W: Widget> Widget for Button<W> { }

impl<W: Widget> Button<W> {
    pub fn new(widget: W, f: impl FnMut(&mut W, Input) -> Damage + 'static) -> Button<W> {
        Button {
            widget: widget,
            callback: Box::new(f),
        }
    }
}
