use std::io::{Write};
use crate::widgets::blend;
use fontconfig::Fontconfig;
use std::fs::read;
use fontdue::{
    Font,
    layout::{
        CoordinateSystem,
        GlyphRasterConfig,
        TextStyle,
        Layout
    },
};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use crate::*;

#[derive(Clone)]
pub struct Label {
    text: Rc<RefCell<Layout>>,
    font_path: PathBuf,
    font: fontdue::Font,
    font_size: f32,
    color: u32,
}

#[derive(Clone)]
pub struct Glyph {
    color: u32,
    config: GlyphRasterConfig,
    bitmap: Vec<u8>,
    metrics: fontdue::Metrics,
}

impl Glyph {
    fn new<'f>(font: &'f Font, config: GlyphRasterConfig, color: u32) -> Glyph {
        let (metrics, bitmap) = font.rasterize_config(config);
        Glyph {
            color,
            config,
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
                        let mut p = [writer[0],writer[1],writer[2],writer[3]];
                        p = blend(&pixel, &p, (255 - *t) as f32 / 255.0);
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
    fn resize(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        Err(Error::Dimension("\"label\" cannot be resized", self.get_width(), self.get_height()))
    }
}

impl Widget for Glyph {
    fn send_action<'s>(&'s mut self, _name: Action, _event_loop: &mut Vec<Damage>, _widget_x: u32, _widget_y: u32) {}
}

impl Label {
    pub fn new<'f>(text: &'f str, font: &'f str, font_size: f32, color: u32) -> Label {
        let fc = Fontconfig::new().unwrap();
        let font_path = fc.find(font, None).unwrap().path;
        let font = read(&font_path).unwrap();
        // Parse it into the font type.
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.append(&[&font], &TextStyle::new(text, font_size, 0));
        Label {
            font_path: font_path,
            text: Rc::new(RefCell::new(layout)),
            font,
            font_size,
            color: color,
        }
    }
}

impl Geometry for Label {
    fn get_width(&self) -> u32 {
        if let Some(glyph) = self.text.as_ref().borrow_mut().glyphs().last() {
            (glyph.x + glyph.width as f32) as u32
        } else { 0 }
    }
    fn get_height(&self) -> u32 {
        (self.text.as_ref().borrow().height() * self.text.as_ref().borrow().lines() as f32) as u32
    }
    fn resize(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        Err(Error::Dimension("\"label\" cannot be resized", self.get_width(), self.get_height()))
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
        for glyph_position in self.text.as_ref().borrow_mut().glyphs() {
            let glyph = Glyph::new(&self.font, glyph_position.key, self.color);
            glyph.draw(canvas, width, x + glyph_position.x as u32, y + glyph_position.y as u32);
        }
    }
}

impl Widget for Label {
    fn send_action<'s>(&'s mut self, _name: Action, _event_loop: &mut Vec<Damage>, _widget_x: u32, _widget_y: u32) {}
}
