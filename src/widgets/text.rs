use crate::*;
use widgets::u32_to_source;
use scene::*;
pub use fontdue::{
    layout,
    layout::{
        CoordinateSystem, GlyphPosition, GlyphRasterConfig, Layout, LayoutSettings, TextStyle,
    },
    Font, FontResult, FontSettings,
};
use raqote::*;
use std::clone::Clone;
use std::collections::HashMap;
use std::fs::read;
use std::path::Path;

pub fn font_from_path(path: &Path) -> Font {
    let font = read(path).unwrap();
    Font::from_bytes(font, fontdue::FontSettings::default()).unwrap()
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
    pub fn render_glyph(&mut self, glyph: &GlyphPosition, source: SolidSource) -> Option<Vec<u32>> {
        if !glyph.char_data.is_missing() {
            let pixmap: Vec<u32>;
            if let Some(coverage) = self.glyphs.get(&glyph.key) {
                pixmap = coverage
                    .iter()
                    .map(|a| {
                        if a == &0 {
                            0
                        } else {
                            SolidSource::from_unpremultiplied_argb(*a, source.r, source.g, source.b)
                                .to_u32()
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
                            SolidSource::from_unpremultiplied_argb(*a, source.r, source.g, source.b)
                                .to_u32()
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

#[derive(Clone, Debug)]
pub struct LabelData {
    text: String,
    font_size: f32,
    pub fonts: Vec<String>,
    pub source: SolidSource,
    pub glyphs: Vec<GlyphPosition>,
}

impl LabelData {
    fn default() -> LabelData {
        LabelData {
            text: String::new(),
            font_size: 0.,
            fonts: Vec::new(),
            source: u32_to_source(0),
            glyphs: Vec::new(),
        }
    }
}

impl PartialEq for LabelData {
    fn eq(&self, other: &Self) -> bool {
        self.text.eq(&other.text)
        && self.font_size == other.font_size
        && self.fonts.eq(&other.fonts)
        && self.source == other.source
    }
}

pub struct Label {
    data: LabelData,
    width: f32,
    layout: Layout,
    write_buffer: Option<String>,
    settings: LayoutSettings,
}

impl Geometry for Label {
    fn width(&self) -> f32 {
        if let Some(width) = self.settings.max_width {
            width
        } else {
            self.width
        }
    }
    fn height(&self) -> f32 {
        if let Some(height) = self.settings.max_height {
            height
        } else {
            self.layout.height()
        }
    }
}

impl Clone for Label {
    fn clone(&self) -> Self {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&self.settings);
        Label {
            layout,
            data: self.data.clone(),
            width: self.width,
            write_buffer: self.write_buffer.clone(),
            settings: self.settings,
        }
    }
}

impl Drawable for Label {
    fn set_color(&mut self, color: u32) {
        let color = color.to_be_bytes();
        self.data.source = SolidSource {
            a: color[0],
            r: color[1],
            g: color[2],
            b: color[3],
        };
    }
    fn draw(&self, ctx: &mut Context, x: f32, y: f32) {
        ctx.draw_label(x, y, &self.data);
    }
}

fn create_layout(max_width: Option<f32>, max_height: Option<f32>) -> (LayoutSettings, Layout) {
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

fn get_size<U: Copy + Clone>(glyphs: &Vec<GlyphPosition<U>>) -> (f32, f32) {
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

fn create_source(color: u32) -> SolidSource {
    let color = color.to_be_bytes();
    SolidSource {
        a: color[0],
        r: color[1],
        g: color[2],
        b: color[3],
    }
}

impl Label {
    pub fn new(text: &str, font: &str, font_size: f32, color: u32) -> Label {
        let (settings, mut layout) = create_layout(None, None);
        let (width, _) = get_size(layout.glyphs());
        let mut data = LabelData::default();
        data.font_size = font_size;
        data.fonts = vec![font.to_owned()];
        data.source = u32_to_source(color);
        data.glyphs = layout.glyphs().clone();
        Label {
            data,
            width,
            settings,
            write_buffer: Some(text.to_owned()),
            layout,
        }
    }
    pub fn new_with_size(
        text: &str,
        font: &str,
        font_size: f32,
        width: f32,
        height: f32,
        color: u32,
    ) -> Label {
        let (settings, mut layout) = create_layout(Some(width), Some(height));
        let mut data = LabelData::default();
        data.font_size = font_size;
        data.glyphs = layout.glyphs().clone();
        data.fonts = vec![font.to_owned()];
        data.source = u32_to_source(color);
        Label {
            data,
            width,
            settings,
            write_buffer: Some(text.to_owned()),
            layout,
        }
    }
    pub fn max_width(text: &str, font: &str, font_size: f32, width: f32, color: u32) -> Label {
        let (settings, mut layout) = create_layout(Some(width), None);
        let mut data = LabelData::default();
        data.font_size = font_size;
        data.glyphs = layout.glyphs().clone();
        data.fonts = vec![font.to_owned()];
        data.source = u32_to_source(color);
        Label {
            data,
            width,
            settings,
            write_buffer: Some(text.to_owned()),
            layout,
        }
    }
    pub fn add_font(&mut self, font: &str) {
        self.data.fonts.push(font.to_string());
    }
    pub fn write(&mut self, text: &str) {
        self.data.text = text.to_string();
        if let Some(buffer) = self.write_buffer.as_mut() {
            buffer.push_str(text);
        } else {
            self.write_buffer = Some(text.to_owned());
        }
    }
    pub fn edit(&mut self, text: &str) {
        self.data.text = text.to_string();
        self.write_buffer = Some(text.to_owned());
        self.layout.clear();
        self.layout.reset(&self.settings);
    }
}


impl Widget for Label {
    fn create_node(&self, x: f32, y: f32) -> RenderNode {
        RenderNode::Widget(Damage::from_text(
            Region::new(x, y, self.width(), self.height()),
            self.data.clone()
        ))
    }
    fn roundtrip<'d>(&'d mut self, _wx: f32, _wy: f32, ctx: &mut Context, _dispatch: &Dispatch) {
        if let Some(text) = self.write_buffer.as_ref() {
            let fonts = ctx.get_fonts(&self.data.fonts);
            if !fonts.is_empty() {
                for c in text.chars() {
                    for (i, font) in fonts.iter().enumerate() {
                        if font.lookup_glyph_index(c) != 0 {
                            self.layout
                                .append(&fonts, &TextStyle::new(&c.to_string(), self.data.font_size, i));
                            break;
                        }
                    }
                }
                let (width, _) = get_size(&mut self.layout.glyphs());
                self.width = width;
                self.data.glyphs = self.layout.glyphs().clone();
                self.write_buffer = None;
            }
        }
    }
}
