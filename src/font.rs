use std::io::{Write};
use crate::widgets::{render, Surface, Wbox, Alignment, Rectangle, blend};
use fontconfig::{Font, Fontconfig};
use std::fs::read;
use std::path::PathBuf;
use crate::*;

#[derive(Clone)]
pub struct Label {
    text: Wbox,
    font_path: PathBuf,
    font: fontdue::Font,
    font_size: f32,
    color: u32,
}

#[derive(Clone)]
pub struct Glyph {
    color: u32,
    glyph: char,
    bitmap: Vec<u8>,
    metrics: fontdue::Metrics,
}

impl Glyph {
    fn new<'f>(glyph: char, font: &'f fontdue::Font, font_size: f32, color: u32) -> Glyph {
        let (metrics, bitmap) = font.rasterize(glyph, font_size);
        Glyph {
            glyph,
            color,
            bitmap,
            metrics,
        }
    }
}

impl Drawable for Glyph {
    fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        let size = canvas.len();
        let mut index = ((x + (y * width as u32)) * 4) as usize;
        if index < size {
            let mut writer = &mut canvas[index..];
            for (i, t) in self.bitmap.iter().enumerate() {
                let pixel = self.color.to_ne_bytes();
                match t {
                    &0 => {
                        let p = [writer[0],writer[1],writer[2],writer[3]];
                        writer.write(&p).unwrap();
                    }
                    &255 => {
                        writer.write(&pixel).unwrap();
                    }
                    _ => if i < writer.len() {
                        let p = blend(&writer[i..], &pixel, *t as i32);
                        writer.write(&p).unwrap();
                    }
                }
                if (i + 1) % self.metrics.width == 0 {
                    index += width as usize * 4;
                    writer.flush().unwrap();
                    writer = &mut canvas[index..];
                }
            }
        }
    }
}

impl Geometry for Glyph {
    fn get_width(&self) -> u32 {
        self.metrics.width as u32
    }
    fn get_height(&self) -> u32 {
        self.metrics.height as u32
    }
    fn contains(&mut self, _widget_x: u32, _widget_y: u32, _x: u32, _y: u32, _event: Input) -> Damage {
        Damage::None
    }
}

impl Widget for Glyph {}

impl Label {
    pub fn new<'f>(text: &'f str, font: &'f str, font_size: f32, color: u32) -> Label {
        let fc = Fontconfig::new().unwrap();
        let font_path = fc.find(font, None).unwrap().path;
        let font = read(&font_path).unwrap();
        // Parse it into the font type.
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut wbox = Wbox::new();
        for c in text.chars() {
            let glyph = Glyph::new(c, &font, font_size, color);
            let _ = if c == ' ' {
                wbox.add(Rectangle::empty(glyph.metrics.advance_width as u32, 0))
            } else {
                /*
                let (dx, dy) = (
                    glyph.metrics.xmin,
                    glyph.metrics.ymin,
                );
                let (x, y) = (
                    glyph.metrics.advance_width,
                    glyph.metrics.advance_height,
                );
                */
                let e = wbox.add(glyph);
                /*
                let width = wbox.get_width();
                let g = wbox.widgets.last_mut().unwrap();
                if dx > 0 {
                    g.translate(dx.abs() as u32 + x as u32, 0);
                }
                if dy > 0 {
                    g.translate(0, dy.abs() as u32 + y as u32);
                }
                */
                e
            };
        }
        wbox.justify(Alignment::End);
        Label {
            font_path: font_path,
            text: wbox,
            font,
            font_size,
            color: color,
        }
    }
    pub fn write(&mut self, text: &'static str) {
        let font = read(&self.font_path).unwrap();
        // Parse it into the font type.
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        self.text = Wbox::new();
        for c in text.chars() {
            self.text.add(Glyph::new(c, &font, self.font_size, self.color)).unwrap();
        }
    }
}

impl Geometry for Label {
    fn get_width(&self) -> u32 {
        self.text.get_width()
    }
    fn get_height(&self) -> u32 {
        self.text.get_height()
    }
    fn contains(&mut self, _widget_x: u32, _widget_y: u32, _x: u32, _y: u32, _event: Input) -> Damage {
        Damage::None
    }
}

impl Drawable for Label {
    fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.text.draw(canvas, width, x, y);
    }
}

impl Widget for Label {}
