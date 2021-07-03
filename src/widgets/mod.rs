pub mod list;
pub mod wbox;
pub mod button;

use crate::snui::*;
pub use wbox::Wbox as Wbox;
pub use list::List as List;
pub use button::Button as Button;

#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub empty: bool,
    content: Content,
    width: u32,
    height: u32,
}

impl Drawable for Rectangle {
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        self.height
    }
    fn set_content(&mut self, content: Content) {
        self.content = content;
    }
    fn contains(&mut self, x: u32, y: u32, event: Input) -> bool {
        true
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
                    canvas.set(x+dx, y+dy, self.content);
                } else {
                    canvas.set(x+dx, y+dy, self.content);
                }
            }
        }
    }
}

impl Rectangle {
    pub fn new(width: u32, height: u32) -> Rectangle {
        Rectangle {
            content: Content::Empty,
            width,
            height,
            empty: false,
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

// A minimal implementation of a canvas objects can use to draw themselves
pub struct Surface {
    width: u32,
    height: u32,
    canvas: Vec<Content>,
}

impl Drawable for Surface {
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        self.height
    }
    fn set_content(&mut self, content: Content) {}
    fn contains(&mut self, x: u32, y: u32, event: Input) -> bool {
        true
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
        canvas.composite(self, x, y);
    }
}

impl Canvas for Surface {
    fn paint(&self) {}
    fn get(&self, x: u32, y: u32) -> Content {
        let index = (x + (y * self.get_width()));
        self.canvas[index as usize]
    }
    fn damage(&mut self, event: Damage) {
        match event {
            Damage::All{ surface } => {
                self.composite(&surface, 0, 0);
            }
            Damage::Area{ surface, x, y } => {
                self.composite(&surface, x, y);
            }
            Damage::Destroy{ x, y, width, height } => {
                for x in x..x+width {
                    for x in y..y+height {
                        self.set(x, y, Content::Empty);
                    }
                }
            }
            Damage::None => {}
        }
    }
    fn composite(&mut self, surface: &Surface, x: u32, y: u32) {
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
            let y = self.height - y - 1;
            let index = (x + (y * self.get_width()));
            self.canvas[index as usize] = content;
        }
    }
}

impl Surface {
    pub fn empty(width: u32, height: u32) -> Surface {
        let canvas = vec![Content::Empty; (width * height) as usize];
        Surface {
            width: width,
            height: height,
            canvas,
        }
    }
    pub fn new(width: u32, height: u32, content: Content) -> Surface {
        let canvas = vec![content; (width * height) as usize];
        Surface {
            width,
            height,
            canvas,
        }
    }
}

pub fn anchor<D, C>(surface: &mut Surface, geometry: &D, anchor: Anchor, margin: u32)
where
    D: Drawable,
{
    if surface.get_width() >= geometry.get_width() && surface.get_height() >= geometry.get_height()
    {
        let mut x = (surface.get_width() - geometry.get_width()) / 2;
        let mut y = (surface.get_height() - geometry.get_height()) / 2;
        match anchor {
            Anchor::Left => x = margin,
            Anchor::Right => x = surface.get_width() - geometry.get_width() - margin,
            Anchor::Top => y = margin,
            Anchor::Bottom => y = surface.get_height() - geometry.get_height() - margin,
            Anchor::Center => {}
            Anchor::TopRight => {
                x = surface.get_width() - geometry.get_width() - margin;
                y = surface.get_height() - geometry.get_height() - margin;
            }
            Anchor::TopLeft => {
                x = margin;
                y = surface.get_height() - geometry.get_height() - margin;
            }
            Anchor::BottomRight => {
                x = surface.get_width() - geometry.get_width() - margin;
                y = margin;
            }
            Anchor::BottomLeft => {
                x = margin;
                y = margin;
            }
        }
        geometry.draw(surface, x, y);
    } else {
        // TO-DO
        // Actually use the Error enum
        print!("Requested size: {} x {}\n", geometry.get_width(), geometry.get_height());
        print!("Available size: {} x {}\n", surface.get_width(), surface.get_height());
        println!("widget doesn't fit on the surface");
    }
}
