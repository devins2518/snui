pub mod button;
pub mod inner;
pub mod listbox;
pub mod revealer;
pub mod wbox;

use crate::snui::*;
pub use button::Button;
pub use revealer::Revealer;
pub use inner::Inner;
pub use listbox::ListBox;
pub use wbox::Wbox;

const TRANSPARENT: u32 = 0x00_00_00_00;

/*
 * The most basic widget one can create. It's the basis of everything else.
 */
#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    width: u32,
    height: u32,
    content: Content,
    pub empty: bool,
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
}

impl Drawable for Rectangle {
    fn set_content(&mut self, content: Content) {
        self.content = content;
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
        for dx in 0..self.get_width() {
            for dy in 0..self.get_height() {
                if self.empty
                    && (dx == 0
                        || dy == 0
                        || dx == self.get_width() - 1
                        || dy == self.get_height() - 1)
                {
                    canvas.set(x + dx, y + dy, self.content);
                } else {
                    canvas.set(x + dx, y + dy, self.content);
                }
            }
        }
    }
}

impl Widget for Rectangle {}

impl Rectangle {
    pub fn new(width: u32, height: u32) -> Rectangle {
        Rectangle {
            content: Content::Empty,
            width,
            height,
            empty: false,
        }
    }
    pub fn empty(width: u32, height: u32) -> Rectangle {
        Rectangle {
            content: Content::Empty,
            width,
            height,
            empty: true,
        }
    }
    pub fn square(size: u32, content: Content) -> Rectangle {
        Rectangle {
            content,
            width: size,
            height: size,
            empty: false,
        }
    }
}

// A minimal implementation of a canvas widgets can use to draw themselves
#[derive(Clone, Debug)]
pub struct Surface {
    width: u32,
    height: u32,
    canvas: Vec<u8>,
}

impl Geometry for Surface {
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
}

impl Canvas for Surface {
    fn display(&mut self) {}
    fn get(&self, x: u32, y: u32) -> Content {
        let index = x + (y * self.get_width());
        Content::Byte(self.canvas[index as usize])
    }
    fn damage(&mut self, event: Damage) {
        match event {
            Damage::All { surface } => {
                self.composite(&surface, 0, 0);
            }
            Damage::Area { surface, x, y } => {
                self.composite(&surface, x, y);
            }
            Damage::Destroy {
                x,
                y,
                width,
                height,
            } => {
                for x in 0..x + width {
                    for y in 0..y + height {
                        self.set(x, y, Content::Empty);
                    }
                }
            }
            _ => {}
        }
    }
    fn composite(&mut self, surface: &(impl Canvas + Geometry), x: u32, y: u32) {
        let width = if x + surface.get_width() <= self.width {
            surface.get_width()
        } else if self.width > x {
            self.width - x
        } else {
            0
        };
        let height = if y + surface.get_height() <= self.height {
            surface.get_height()
        } else if self.height > y {
            self.height - y
        } else {
            0
        };
        for dx in 0..width {
            for dy in 0..height {
                let content = surface.get(dx, dy);
                self.set(x + dx, y + dy, content);
            }
        }
    }
    fn set(&mut self, x: u32, y: u32, content: Content) {
        if ((x * y) as usize) < self.canvas.len() {
            match content {
                Content::Pixel(pixel) => {
                    self.set_pixel(x, y, pixel);
                }
                Content::Byte(byte) => {
                    let index = x + (y * self.get_width());
                    self.canvas[index as usize] = byte;
                }
                _ => {}
            }
        }
    }
}

impl Surface {
    pub fn new(width: u32, height: u32) -> Surface {
        let canvas = vec![0; (width * height * 4) as usize];
        Surface {
            width: width,
            height: height,
            canvas,
        }
    }
    fn set_pixel(&mut self, x: u32, y: u32, pixel: u32) {
        let mut index = ((x + (y * self.get_width())) * 4) as usize;
        for byte in &pixel.to_ne_bytes() {
            self.canvas[index as usize] = *byte;
            index += 1;
        }
    }
    pub fn get_buf(&mut self) -> &mut [u8] {
        &mut self.canvas
    }
}

pub fn anchor<W: 'static, C>(
    container: &mut C,
    widget: W,
    anchor: Anchor,
    margin: u32,
) -> Result<(), Error>
where
    W: Widget,
    C: Container + Geometry,
{
    if container.get_width() >= widget.get_width() && container.get_height() >= widget.get_height()
    {
        let mut x = (container.get_width() - widget.get_width()) / 2;
        let mut y = (container.get_height() - widget.get_height()) / 2;
        match anchor {
            Anchor::Left => x = margin,
            Anchor::Right => x = container.get_width() - widget.get_width() - margin,
            Anchor::Top => y = margin,
            Anchor::Bottom => y = container.get_height() - widget.get_height() - margin,
            Anchor::Center => {}
            Anchor::TopRight => {
                x = container.get_width() - widget.get_width() - margin;
                y = container.get_height() - widget.get_height() - margin;
            }
            Anchor::TopLeft => {
                x = margin;
                y = container.get_height() - widget.get_height() - margin;
            }
            Anchor::BottomRight => {
                x = container.get_width() - widget.get_width() - margin;
                y = margin;
            }
            Anchor::BottomLeft => {
                x = margin;
                y = margin;
            }
        }
        container.put(Inner::new_at(widget, x, y))
    } else {
        Err(Error::Dimension(
            "anchor",
            widget.get_width(),
            widget.get_height(),
        ))
    }
}
