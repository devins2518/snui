use crate::*;
use crate::widgets::*;

pub struct Background<B: Widget, W: Widget> {
    pub widget: W,
    pub background: B,
    padding: (u32, u32, u32, u32),
}

impl<B: Widget, W: Widget> Geometry for Background<B, W> {
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

impl<B: Widget, W: Widget> Drawable for Background<B, W> {
    fn set_color(&mut self, color: u32) {
        self.background.set_color(color);
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.background.draw(canvas, width, x, y);

        self.widget
            .draw(canvas, width, x + self.padding.3, y + self.padding.0);
    }
}

impl<B: Widget, W: Widget> Widget for Background<B, W> {
    fn send_command<'s>(
        &'s mut self,
        command: Command,
        damage_queue: &mut Vec<Damage<'s>>,
        x: u32,
        y: u32,
    ) {
        self.background.send_command(command, damage_queue, x, y);
        self.widget
            .send_command(command, damage_queue, x + self.padding.0, y + self.padding.3)
    }
}

impl<B: Widget, W: Widget> Background<B, W> {
    pub fn new(widget: W, mut background: B, padding: u32) -> Self {
        let width = widget.get_width() + 2 * padding;
        let height = widget.get_height() + 2 * padding;
        // Might potentially correct
        background.resize(width, height).unwrap();
        Self {
            widget: widget,
            background: background,
            padding: (padding, padding, padding, padding),
        }
    }
    pub fn solid(widget: W, color: u32, padding: u32) -> Background<Rectangle, W> {
        let width = widget.get_width() + 2 * padding;
        let height = widget.get_height() + 2 * padding;
        let background = Rectangle::new(width, height, color);
        Background {
            widget: widget,
            background: background,
            padding: (padding, padding, padding, padding),
        }
    }
    pub fn set_padding(&mut self, top: u32, right: u32, bottom: u32, left: u32) {
        let width = self.widget.get_width() + left + right;
        let height = self.widget.get_height() + top + bottom;
        // Might potentially correct
        self.background.resize(width, height).unwrap();
        self.padding = (top, right, bottom, left);
    }
}
