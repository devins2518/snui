use crate::widgets::text::Label;
pub use fontdue::{
    layout,
    layout::{
        CoordinateSystem, GlyphPosition, GlyphRasterConfig, Layout, LayoutSettings, TextStyle,
    },
    Font, FontResult, FontSettings,
};
use std::clone::Clone;
use std::collections::HashMap;
use std::fs::read;
use std::path::Path;
use tiny_skia::*;

pub fn font_from_path(path: &Path) -> Font {
    let font = read(path).unwrap();
    Font::from_bytes(font, fontdue::FontSettings::default()).unwrap()
}

pub fn create_layout(max_width: Option<f32>, max_height: Option<f32>) -> (LayoutSettings, Layout) {
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    let setting = LayoutSettings {
        x: 0.,
        y: 0.,
        max_width,
        max_height,
        horizontal_align: layout::HorizontalAlign::Left,
        vertical_align: layout::VerticalAlign::Middle,
        wrap_style: layout::WrapStyle::Word,
        wrap_hard_breaks: true,
    };
    layout.reset(&setting);
    (setting, layout)
}

pub fn get_size<U: Copy + Clone>(glyphs: &Vec<GlyphPosition<U>>) -> (f32, f32) {
    let mut width = 0;
    let mut height = 0;
    for gp in glyphs {
        if width < gp.width + gp.x as usize {
            width = gp.width + gp.x as usize
        }
        if height < gp.height + gp.y as usize {
            height = gp.height + gp.y as usize
        }
    }
    (width as f32, height as f32)
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FontProperty {
    pub name: String,
}

impl FontProperty {
    pub fn new(name: &str) -> FontProperty {
        FontProperty {
            name: name.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct FontCache {
    pub fonts: HashMap<FontProperty, GlyphCache>,
}

impl FontCache {
    pub fn new() -> Self {
        FontCache {
            fonts: HashMap::new(),
        }
    }
    pub fn get_fonts(&self, fonts: &[FontProperty]) -> Vec<&Font> {
        fonts
            .iter()
            .filter_map(|font| {
                if let Some(glyph_cache) = self.fonts.get(font) {
                    return Some(&glyph_cache.font);
                }
                None
            })
            .collect()
    }
    pub fn load_font(&mut self, name: &str, path: &std::path::Path) {
        if let Ok(glyph_cache) = GlyphCache::load(path) {
            self.fonts.insert(
                FontProperty {
                    name: name.to_owned(),
                },
                glyph_cache,
            );
        } else {
            eprintln!("No font at {:?}", path);
        }
    }
    pub fn write(&mut self, layout: &mut Layout, label: &Label, _string: &str) {
        let fonts = self.get_fonts(label.fonts());
        for c in label.text().chars() {
            for (i, font) in fonts.iter().enumerate() {
                if font.lookup_glyph_index(c) != 0 {
                    layout.append(
                        &fonts,
                        &TextStyle::new(&c.to_string(), label.font_size(), i),
                    );
                    break;
                }
            }
        }
    }
    pub fn layout(&mut self, label: &Label) -> Layout {
        let fonts = self.get_fonts(label.fonts());
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(label.settings());
        for c in label.text().chars() {
            for (i, font) in fonts.iter().enumerate() {
                if font.lookup_glyph_index(c) != 0 {
                    layout.append(
                        &fonts,
                        &TextStyle::new(&c.to_string(), label.font_size(), i),
                    );
                    break;
                }
            }
        }
        layout
    }
}

#[derive(Debug, Clone)]
pub struct GlyphCache {
    pub font: Font,
    glyphs: HashMap<GlyphRasterConfig, Vec<u8>>,
}

impl GlyphCache {
    pub fn new(font: Font) -> Self {
        Self {
            font,
            glyphs: HashMap::new(),
        }
    }
    pub fn load(path: &Path) -> FontResult<Self> {
        if let Ok(bytes) = read(path) {
            if let Ok(font) = Font::from_bytes(bytes, fontdue::FontSettings::default()) {
                Ok(Self {
                    font,
                    glyphs: HashMap::new(),
                })
            } else {
                FontResult::Err("Isn't a font")
            }
        } else {
            FontResult::Err("Invalid path")
        }
    }
    pub fn render_glyph(&mut self, glyph: &GlyphPosition, source: Color) -> Option<Vec<u32>> {
        if !glyph.char_data.is_missing() {
            let pixmap: Vec<u32>;
            if let Some(coverage) = self.glyphs.get(&glyph.key) {
                pixmap = coverage
                    .iter()
                    .map(|a| {
                        if a == &0 {
                            0
                        } else {
                            let mut color = source;
                            color.apply_opacity(*a as f32 / 255.);
                            color.premultiply().to_color_u8().get()
                        }
                    })
                    .collect();
            } else {
                let (_, coverage) = self.font.rasterize_config(glyph.key);
                pixmap = coverage
                    .iter()
                    .map(|a| {
                        if a == &0 {
                            0
                        } else {
                            let mut color = source;
                            color.apply_opacity(*a as f32 / 255.);
                            color.premultiply().to_color_u8().get()
                        }
                    })
                    .collect();
                self.glyphs.insert(glyph.key, coverage);
            }
            return Some(pixmap);
        }
        None
    }
}
