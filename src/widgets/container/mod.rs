pub mod layout;
pub mod revealer;
pub mod wbox;
use crate::widgets::*;
use crate::*;

pub use layout::{Alignment, WidgetLayout};
pub use revealer::Revealer;
pub use wbox::Wbox;

pub struct Border<W: Widget> {
    pub widget: W,
    color: u32,
    damaged: bool,
    size: (u32, u32, u32, u32),
}

impl<W: Widget> Geometry for Border<W> {
    fn get_width(&self) -> u32 {
        self.widget.get_width() + self.size.0 + self.size.2
    }
    fn get_height(&self) -> u32 {
        self.widget.get_height() + self.size.1 + self.size.3
    }
}

impl<W: Widget> Drawable for Border<W> {
    fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    fn draw(&self, canvas: &mut Canvas, x: u32, y: u32) {
        let bwidth = self.get_width();
        let bheight = self.get_height();

        Rectangle::new(bwidth, self.size.0, self.color).draw(canvas, x, y);
        Rectangle::new(bwidth, self.size.2, self.color).draw(canvas, x, y + bheight - self.size.2);
        Rectangle::new(self.size.1, bheight, self.color).draw(canvas, x + bwidth - self.size.1, y);
        Rectangle::new(self.size.3, bheight, self.color).draw(canvas, x, y);

        self.widget.draw(canvas, x + self.size.0, y + self.size.3);
    }
}

impl<W: Widget> Widget for Border<W> {
    fn damaged(&self) -> bool {
        self.damaged
    }
    fn roundtrip<'d>(
        &'d mut self,
        widget_x: u32,
        widget_y: u32,
        dispatched: &Dispatch,
    ) -> Option<Damage> {
        self.widget
            .roundtrip(widget_x + self.size.0, widget_y + self.size.3, dispatched)
    }
}

impl<W: Widget> Border<W> {
    pub fn new(widget: W, size: u32, color: u32) -> Self {
        Self {
            widget,
            color,
            damaged: true,
            size: (size, size, size, size),
        }
    }
    pub fn set_border_size(&mut self, top: u32, right: u32, bottom: u32, left: u32) {
        self.size = (top, right, bottom, left);
    }
}

pub struct Background<W: Widget> {
    pub widget: W,
    damaged: bool,
    pub background: u32,
    padding: (u32, u32, u32, u32),
}

impl<W: Widget> Geometry for Background<W> {
    fn get_width(&self) -> u32 {
        self.widget.get_width() + self.padding.1 + self.padding.3
    }
    fn get_height(&self) -> u32 {
        self.widget.get_height() + self.padding.0 + self.padding.2
    }
}

impl<W: Widget> Drawable for Background<W> {
    fn set_color(&mut self, color: u32) {
        self.background = color;
    }
    fn draw(&self, canvas: &mut Canvas, x: u32, y: u32) {
        Rectangle::new(self.get_width(), self.get_height(), self.background).draw(canvas, x, y);

        self.widget
            .draw(canvas, x + self.padding.3, y + self.padding.0);
    }
}

impl<W: Widget> Widget for Background<W> {
    fn damaged(&self) -> bool {
        self.damaged
    }
    fn roundtrip<'d>(
        &'d mut self,
        widget_x: u32,
        widget_y: u32,
        dispatched: &Dispatch,
    ) -> Option<Damage> {
        self.widget.roundtrip(
            widget_x + self.padding.3,
            widget_y + self.padding.0,
            dispatched,
        )
    }
}

impl<W: Widget> Background<W> {
    pub fn new(widget: W, color: u32, padding: u32) -> Background<W> {
        Background {
            widget: widget,
            damaged: true,
            background: color,
            padding: (padding, padding, padding, padding),
        }
    }
    pub fn set_padding(&mut self, top: u32, right: u32, bottom: u32, left: u32) {
        self.padding = (top, right, bottom, left);
    }
}
