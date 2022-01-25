use crate::widgets::text::Label;
use fontconfig::Fontconfig;
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

pub fn get_size<U: Copy + Clone>(glyphs: &Vec<GlyphPosition<U>>) -> (f32, f32) {
    let mut width = 0;
    let mut height = 0;
    for gp in glyphs {
        width = width.max(gp.width + gp.x as usize);
        height = height.max(gp.height + gp.y as usize)
    }
    (width as f32, height as f32)
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FontStyle {
    Regular,
    Italic,
    Bold,
}

impl FontStyle {
    fn as_str(&self) -> Option<&str> {
        match self {
            Self::Regular => None,
            Self::Italic => Some("italic"),
            Self::Bold => Some("bold"),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FontProperty {
    pub name: String,
    pub style: FontStyle,
}

impl From<&str> for FontProperty {
    fn from(name: &str) -> Self {
        FontProperty {
            name: name.to_string(),
            style: FontStyle::Regular,
        }
    }
}

impl FontProperty {
    pub fn new(name: &str) -> FontProperty {
        FontProperty {
            name: name.to_string(),
            style: FontStyle::Regular,
        }
    }
}

/// A fontconfig backed FontCache.
/// If a font cannot be found it will load it if possible.
pub struct FontCache {
    pub(crate) fc: Option<Fontconfig>,
    pub(crate) layout: Layout,
    pub(crate) fonts: HashMap<FontProperty, GlyphCache>,
}

impl FontCache {
    pub fn new() -> Self {
        FontCache {
            fc: Fontconfig::new(),
            fonts: HashMap::new(),
            layout: Layout::new(CoordinateSystem::PositiveYDown),
        }
    }
    pub fn get_fonts<'f>(
        cache: &'f HashMap<FontProperty, GlyphCache>,
        fonts: &[FontProperty],
    ) -> Vec<&'f Font> {
        fonts
            .iter()
            .filter_map(|font| {
                if let Some(glyph_cache) = cache.get(font) {
                    return Some(&glyph_cache.font);
                }
                None
            })
            .collect()
    }
    pub fn load_font(&mut self, config: &FontProperty) {
        if self.fonts.get(&config).is_none() {
            if let Some(fc) = self.fc.as_ref() {
                if let Some(fc_font) = fc.find(&config.name, config.style.as_str()) {
                    match GlyphCache::load(fc_font.path.as_path()) {
                        Ok(glyph_cache) => {
                            self.fonts.insert(config.clone(), glyph_cache);
                        }
                        Err(e) => {
                            eprintln!("{}: {:?}", e, config);
                        }
                    }
                }
            }
        }
    }
    pub fn write(&mut self, layout: &mut Layout, label: &Label, string: &str) {
        for font in &label.fonts {
            self.load_font(font);
        }
        let fonts = Self::get_fonts(&self.fonts, &label.fonts);
        for c in string.chars() {
            for (i, font) in fonts.iter().enumerate() {
                if font.lookup_glyph_index(c) != 0 {
                    layout.append(
                        &fonts,
                        &TextStyle::new(&c.to_string(), label.get_font_size(), i),
                    );
                    break;
                }
            }
        }
    }
    pub fn layout(&mut self, label: &Label) -> &Vec<GlyphPosition> {
        for font in &label.fonts {
            self.load_font(font);
        }
        let fonts = Self::get_fonts(&self.fonts, &label.fonts);
        self.layout.reset(&label.settings);
        for c in label.get_text().chars() {
            for (i, font) in fonts.iter().enumerate() {
                if font.lookup_glyph_index(c) != 0 {
                    self.layout.append(
                        &fonts,
                        &TextStyle::new(&c.to_string(), label.get_font_size(), i),
                    );
                    break;
                }
            }
        }
        self.layout.glyphs()
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
        match read(path) {
            Ok(bytes) => match Font::from_bytes(bytes, fontdue::FontSettings::default()) {
                Ok(font) => Ok(Self {
                    font,
                    glyphs: HashMap::new(),
                }),
                Err(_) => FontResult::Err("Isn't a font"),
            },
            Err(_) => FontResult::Err("Invalid path"),
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
