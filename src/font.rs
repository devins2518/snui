use std::io::Read;
use crate::widgets::{render, ListBox, Surface};
use crate::*;

pub struct Label {
    text: &'static str,
    font: &'static str,
    font_size: f32,
    color: u32,
}

impl Label {
    pub fn new(text: &'static str, font: &'static str, font_size: f32) -> Label {
        Label {
            text,
            font,
            font_size,
            color: 0xD0_FF_00_00,
        }
    }
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
    }
    pub fn set_font(&mut self, font: &'static str) {
        self.font = font;
    }
}

impl Geometry for Label {
    fn get_width(&self) -> u32 {
        ((self.text.len() as f32) * self.font_size) as u32
    }
    fn get_height(&self) -> u32 {
        self.font_size.ceil() as u32
    }
    fn contains(&mut self, _widget_x: u32, _widget_y: u32, _x: u32, _y: u32, _event: Input) -> Damage {
        Damage::None
    }
}

impl Drawable for Label {
    fn set_content(&mut self, content: Content) {
        if let Content::Pixel(color) = content {
            self.color = color;
        }
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        let font = include_bytes!("/home/bryan/.local/share/fonts/TerminusTTF-Bold.ttf") as &[u8];
        // Parse it into the font type.
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut fmt = ListBox::new(Orientation::Horizontal);
        for c in self.text.chars() {
            let (metrics, bitmap) = font.rasterize(c, self.font_size);
            fmt.add(Surface::from(bitmap, metrics.width as u32, metrics.height as u32)).unwrap();
        }
        println!("{} {}", fmt.get_width(), fmt.get_height());
        fmt.draw(canvas, width, x, y);
    }
}

impl Widget for Label {}
