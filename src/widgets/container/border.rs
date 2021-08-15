use crate::*;
use crate::widgets::*;

pub struct Border<W: Widget> {
    pub widget: W,
    color: u32,
    size: (u32, u32, u32, u32),
}

impl<W: Widget> Geometry for Border<W> {
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
        event: Event,
    ) -> Damage {
        self.widget
            .contains(widget_x + self.size.0, widget_y + self.size.3, x, y, event)
    }
}

impl<W: Widget> Drawable for Border<W> {
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

impl<W: Widget> Widget for Border<W> {
    fn send_command<'s>(
        &'s mut self,
        command: Command,
        damage_queue: &mut Vec<Damage<'s>>,
        x: u32,
        y: u32,
    ) {
        self.widget
            .send_command(command, damage_queue, x + self.size.0, y + self.size.3);
    }
}

impl<W: Widget> Border<W> {
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
