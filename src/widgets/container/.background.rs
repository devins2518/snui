use crate::*;
use crate::widgets::*;

pub struct Background<W: Widget> {
    pub widget: W,
    pub background: u32,
    padding: (u32, u32, u32, u32),
}

impl<W: Widget> Geometry for Background< W> {
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
        event: Event,
    ) -> Option<Damage> {
        self.widget.contains(
            widget_x + self.padding.3,
            widget_y + self.padding.0,
            x,
            y,
            event,
        )
    }
}

impl<W: Widget> Drawable for Background<W> {
    fn set_color(&mut self, color: u32) {
        self.background = color;
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        Rectangle::new(self.get_width(), self.get_height(), self.background).draw(canvas, width, x, y);

        self.widget
            .draw(canvas, width, x + self.padding.3, y + self.padding.0);
    }
}

impl<W: Widget> Widget for Background<W> { }

impl<W: Widget> Background<W> {
    pub fn new(widget: W, color: u32, padding: u32) -> Background<W> {
        Background {
            widget: widget,
            background: color,
            padding: (padding, padding, padding, padding),
        }
    }
    pub fn set_padding(&mut self, top: u32, right: u32, bottom: u32, left: u32) {
        self.padding = (top, right, bottom, left);
    }
}
