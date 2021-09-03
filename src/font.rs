use crate::widgets::blend;
use crate::*;
use fontdue::{
    layout,
    layout::{CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle},
    Font,
};
use std::path::Path;
use std::fs::read;
use std::io::Write;
use std::sync::{Arc, Mutex};
// use std::path::PathBuf;

pub struct Label {
    color: u32,
    damaged: bool,
    size: (u32, u32),
    font: fontdue::Font,
    layout: Arc<Mutex<Layout>>,
}

#[derive(Debug, Clone)]
pub struct Glyph {
    color: u32,
    position: (u32, u32),
    config: GlyphRasterConfig,
    coverage: Vec<u8>,
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
        let (metrics, coverage) = font.rasterize_config(config);
        Glyph {
            position: (x, y),
            color,
            config,
            coverage,
            metrics,
        }
    }
}

impl Drawable for Glyph {
    fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    fn draw(&self, canvas: &mut Canvas, x: u32, y: u32) {
        let size = canvas.size();
        let stride = canvas.width as usize * 4;
        let mut index =
            (((x + self.position.0) + ((y + self.position.1) * canvas.width as u32)) * 4) as usize;
        if index < size {
            let mut writer = &mut canvas[index..];
            for (i, t) in self.coverage.iter().enumerate() {
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
                    index += stride;
                    writer.flush().unwrap();
                    writer = &mut canvas[index..];
                }
            }
        }
    }
}

impl Label {
    pub fn new(text: &str, path: &Path, font_size: f32) -> Label {
        let font = read(path).unwrap();
        // Parse it into the font type.
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.append(&[&font], &TextStyle::new(text, font_size, 0));
        Label {
            font,
            color: 0,
            damaged: true,
            size: (0, 0),
            layout: Arc::new(Mutex::new(layout)),
        }
    }
    pub fn max_width<'f>(
        text: &'f str,
        path: &Path,
        font_size: f32,
        width: f32,
    ) -> Label {
        let font = read(path).unwrap();
        // Parse it into the font type.
        let mut layout_setting = DEFAULT;
        layout_setting.max_width = Some(width);
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&layout_setting);
        layout.append(&[&font], &TextStyle::new(text, font_size, 0));

        Label {
            font,
            color: 0,
            damaged: true,
            size: (0, 0),
            layout: Arc::new(Mutex::new(layout)),
        }
    }
    pub fn edit<'f>(&mut self, text: &'f str) {
        let font = &self.font;
        if let Ok(mut layout) = self.layout.lock() {
            let font_size = layout.height();
            layout.reset(&DEFAULT);
            layout.append(&[font], &TextStyle::new(text, font_size, 0));
        }
    }
}

impl Geometry for Label {
    fn get_width(&self) -> u32 {
        self.size.0
    }
    fn get_height(&self) -> u32 {
        self.size.1
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.size = (width, height);
        Ok(())
    }
}

impl Drawable for Label {
    fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    fn draw(&self, canvas: &mut Canvas, x: u32, y: u32) {
        if let Ok(mut layout) = self.layout.lock() {
            for glyph in layout.glyphs() {
                Glyph::new(&self.font, glyph.key, self.color, glyph.x as u32, glyph.y as u32).draw(canvas, x, y);
            }
        }
    }
}

impl Widget for Label {
    fn damaged(&self) -> bool {
        self.damaged
    }
    fn roundtrip<'d>(
        &'d mut self,
        _widget_x: u32,
        _widget_y: u32,
        _dispatched: &Dispatch,
    ) -> Option<Damage> {
        None
    }
}
