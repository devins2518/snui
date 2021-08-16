use crate::*;
use crate::widgets::blend;
use fontconfig::Fontconfig;
use fontdue::{
    layout,
    layout::{CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle},
    Font,
};
use std::fs::read;
use std::io::Write;
use std::path::PathBuf;

pub struct Label {
    color: u32,
    width: u32,
    height: u32,
    name: Option<String>,
    layout: Vec<Glyph>,
    font_path: PathBuf,
    font: fontdue::Font,
    font_size: f32,
    fontdue_layout: Layout,
}

#[derive(Debug, Clone)]
pub struct Glyph {
    color: u32,
    position: (u32, u32),
    config: GlyphRasterConfig,
    bitmap: Vec<u8>,
    metrics: fontdue::Metrics,
}

static DEFAULT: LayoutSettings = LayoutSettings {
    x: 0.0,
    y: 0.0,
    max_width: None,
    max_height: None,
    horizontal_align: layout::HorizontalAlign::Left,
    vertical_align: layout::VerticalAlign::Middle,
    wrap_style: layout::WrapStyle::Word,
    wrap_hard_breaks: false,
};

impl Glyph {
    fn new<'f>(font: &'f Font, config: GlyphRasterConfig, color: u32, x: u32, y: u32) -> Glyph {
        let (metrics, bitmap) = font.rasterize_config(config);
        Glyph {
            position: (x, y),
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
        let mut index = (((x + self.position.0) + ((y + self.position.1) * width as u32)) * 4) as usize;
        if index < size {
            let mut writer = &mut canvas[index..];
            for (i, t) in self.bitmap.iter().enumerate() {
                let pixel = self.color.to_ne_bytes();
                match t {
                    &0 => {
                        let p = [writer[0], writer[1], writer[2], writer[3]];
                        writer.write(&p).unwrap();
                    }
                    &255 => {
                        writer.write(&pixel).unwrap();
                    }
                    _ => {
                        if i < writer.len() {
                            let mut p = [writer[0], writer[1], writer[2], writer[3]];
                            p = blend(&pixel, &p, (255 - *t) as f32 / 255.0);
                            writer.write(&p).unwrap();
                        }
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
    fn contains(
        &mut self,
        _widget_x: u32,
        _widget_y: u32,
        _x: u32,
        _y: u32,
        _event: Event,
    ) -> Damage {
        Damage::None
    }
    fn resize(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        Err(Error::Dimension(
            "\"label\" cannot be resized",
            self.get_width(),
            self.get_height(),
        ))
    }
}

impl Widget for Glyph {
    fn send_command<'s>(
        &'s mut self,
        _command: Command,
        _damage_queue: &mut Vec<Damage<'s>>,
        _x: u32,
        _y: u32,
    ) {
    }
}

impl Label {
    pub fn new<'f>(text: &'f str, font: &'f str, font_size: f32, color: u32) -> Label {
        let fc = Fontconfig::new().unwrap();
        let font_path = fc.find(font, None).unwrap().path;
        let font = read(&font_path).unwrap();
        // Parse it into the font type.
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut fontdue_layout = Layout::new(CoordinateSystem::PositiveYDown);
        fontdue_layout.append(&[&font], &TextStyle::new(text, font_size, 0));

        let mut width = 0;
        let height = (font_size * fontdue_layout.lines() as f32) as u32;

        // Getting Glyphs from the Layout
        let layout: Vec<Glyph> = fontdue_layout.glyphs().iter().map(|glyph_position| {
            let delta = glyph_position.x as usize + glyph_position.width;
            if delta > width {
                width = delta;
            }
            Glyph::new(&font, glyph_position.key, color, glyph_position.x as u32, glyph_position.y as u32)
        }).collect();
        Label {
            name: None,
            width: width as u32,
            height,
            fontdue_layout,
            font_path: font_path,
            layout,
            font,
            font_size,
            color: color,
        }
    }
    pub fn new_with_size<'f>(
        text: &'f str,
        font: &'f str,
        font_size: f32,
        color: u32,
        width: f32,
    ) -> Label {
        let fc = Fontconfig::new().unwrap();
        let font_path = fc.find(font, None).unwrap().path;
        let font = read(&font_path).unwrap();
        // Parse it into the font type.
        let mut layout_setting = DEFAULT;
        layout_setting.max_width = Some(width);
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut fontdue_layout= Layout::new(CoordinateSystem::PositiveYDown);
        fontdue_layout.reset(&layout_setting);
        fontdue_layout.append(&[&font], &TextStyle::new(text, font_size, 0));

        let mut width = 0;
        let height = (font_size * fontdue_layout.lines() as f32) as u32;

        // Getting Glyphs from the Layout
        let layout: Vec<Glyph> = fontdue_layout.glyphs().iter().map(|glyph_position| {
            let delta = glyph_position.x as usize + glyph_position.width;
            if delta > width {
                width = delta;
            }
            Glyph::new(&font, glyph_position.key, color, glyph_position.x as u32, glyph_position.y as u32)
        }).collect();
        Label {
            name: None,
            width: width as u32,
            height,
            fontdue_layout,
            font_path: font_path,
            layout,
            font,
            font_size,
            color: color,
        }
    }
    pub fn edit<'f>(&mut self, text: &'f str) {
        let mut width = 0;
        let font = &self.font;
        let color = self.color;
        self.fontdue_layout.reset(&DEFAULT);
        self.fontdue_layout.append(&[&self.font], &TextStyle::new(text, self.font_size, 0));
        self.layout = self.fontdue_layout.glyphs().iter().map(|glyph_position| {
            let delta = glyph_position.x as usize + glyph_position.width;
            if delta > width {
                width = delta;
            }
            Glyph::new(font, glyph_position.key, color, glyph_position.x as u32, glyph_position.y as u32)
        }).collect();
        self.width = width as u32;
    }
    pub fn write<'f>(&mut self, text: &'f str) {
        let font = &self.font;
        let color = self.color;
        self.fontdue_layout.append(&[&self.font], &TextStyle::new(text, self.font_size, 0));
        self.layout = self.fontdue_layout.glyphs().iter().map(|glyph_position| {
            Glyph::new(font, glyph_position.key, color, glyph_position.x as u32, glyph_position.y as u32)
        }).collect();
    }
}

impl Geometry for Label {
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        (self.font_size * self.fontdue_layout.lines() as f32).ceil() as u32
    }
    fn resize(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        Err(Error::Dimension(
            "\"label\" cannot be resized",
            self.get_width(),
            self.get_height(),
        ))
    }
    fn contains(
        &mut self,
        _widget_x: u32,
        _widget_y: u32,
        _x: u32,
        _y: u32,
        _event: Event,
    ) -> Damage {
        Damage::None
    }
}

impl Drawable for Label {
    fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        for glyph in &self.layout {
            glyph.draw(canvas, width, x, y);
        }
    }
}

impl Widget for Label {
    fn send_command<'s>(
        &'s mut self,
        command: Command,
        damage_queue: &mut Vec<Damage<'s>>,
        x: u32,
        y: u32,
    ) {
        if let Some(name) = &self.name {
            if command.eq(name) {
                match command {
                    Command::Key(_, _) => { }
                    _ => if let Some(text) = command.get::<String>() {
                        self.edit(&text);
                        damage_queue.push(Damage::new(self, x, y));
                    }
                }
            }
        }
    }
}
